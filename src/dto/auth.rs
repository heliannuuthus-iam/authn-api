use actix_web::{
    cookie::{time::Duration, Cookie},
    error::{ErrorPreconditionFailed, ErrorUnauthorized},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use http::Uri;
use tracing::error;
use validator::Validate;

use super::{client::ClientIdpConfig, token::Tokens};
use crate::{
    common::{
        cache::redis::{redis_get, redis_setex},
        constant::{
            AuthRequestType, IdpType, PromptType, ResponseType, CONFLICT_RESPONSE_TYPE,
            OPENID_SCOPE,
        },
        errors::{ApiError, Result},
        oauth::{AuthNCodeResponse, OAuthUser, },
        utils::gen_id,
    },
    dto::user::{UserAssociation, UserProfile},
};

#[derive(Debug, Clone, thiserror::Error, serde::Deserialize, serde::Serialize)]
pub enum AuthError {
    #[error("invalid_client")]
    InvalidClient,
    #[error("login_required")]
    LoginRequired,
    #[error("account_selection_required")]
    AccountSelectionRequired,
    #[error("consent_required")]
    ConsentRequired,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Validate, Default)]
pub struct AuthRequest {
    #[validate(length(min = 21, max = 22))]
    pub client_id: String,
    pub connection: IdpType,
    #[validate(length(min = 1, max = 2))]
    pub response_type: Vec<ResponseType>,
    pub scope: Vec<String>,
    pub state: Option<String>,
    #[validate(url)]
    pub redirect_uri: String,
    pub nonce: Option<String>,
    // https://openid.net/specs/openid-connect-core-1_0.html#CodeFlowSteps
    pub prompt: PromptType,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum FlowStage {
    #[default]
    Initialized = 1,
    Authenticating = 2,
    Authenticated = 3,
    Authorized = 4,
    Completed = 5,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Flow {
    pub id: String,
    pub request: AuthRequest,
    pub flow_type: Vec<AuthRequestType>,
    pub client_config: Option<ClientIdpConfig>,
    pub authorization_code: Option<AuthNCodeResponse>,
    pub tokens: Option<Tokens>,
    pub current: Option<UserProfile>,
    pub subject: Option<UserProfile>,
    pub oauth_user: Option<OAuthUser>,
    pub associations: Vec<UserAssociation>,
    pub stage: FlowStage,
    pub error: Option<AuthError>,
    message: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl Flow {
    pub fn gen_id() -> String {
        gen_id(24)
    }

    pub fn new(params: AuthRequest) -> Self {
        Flow {
            id: Self::gen_id(),
            request: params,
            ..Default::default()
        }
    }

    pub fn validate(&mut self) -> Result<()> {
        let redirect_url = &self.client_config.as_ref().unwrap().client.redirect_url;
        // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-security-topics#name-insufficient-redirect-uri-v
        if !redirect_url.contains(&self.request.redirect_uri) {
            return Err(ApiError::ResponseError(ErrorUnauthorized(
                "invalid_redirect_url",
            )));
        };

        if self
            .request
            .response_type
            .iter()
            .filter(|&r| CONFLICT_RESPONSE_TYPE.contains(&r))
            .count()
            == CONFLICT_RESPONSE_TYPE.len()
        {
            return Err(ApiError::ResponseError(ErrorUnauthorized(
                "conflict_response_type",
            )));
        }
        let mut flow_types = &self.flow_type;
        // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequestValidation
        if self.request.scope.contains(&OPENID_SCOPE.to_string()) {
            flow_types.push(AuthRequestType::Oidc);
        }
        flow_types.push(AuthRequestType::Oauth);
        Ok(())
    }

    pub fn next_uri(&self) -> String {
        let mut builder = Uri::builder().scheme("http");
        let next_uri = match self.stage {
            FlowStage::Initialized => "/login",
            FlowStage::Authenticating => "/login",
            FlowStage::Authenticated => "/confirm",
            FlowStage::Authorized => self.request.redirect_uri.as_str(),
            FlowStage::Completed => "/done",
        };
        builder = builder.path_and_query(next_uri);
        if let Some(auth_error) = &self.error {
            builder.path_and_query(format!("error={auth_error}"))
        } else {
            builder
        }
        .build()
        .unwrap_or_default()
        .to_string()
    }

    pub fn dispatch(&self) -> Result<HttpResponse> {
        let mut resp = match self.stage {
            FlowStage::Initialized | FlowStage::Authenticating => {
                // 展示认证和登录页面，让用户继续流程（可能输入用户名和密码也可能输入验证码等）
                HttpResponse::MultipleChoices()
                    .append_header((http::header::LOCATION, self.next_uri()))
                    .finish()
            }
            FlowStage::Authenticated => {
                // 已认证，需要授权
                HttpResponse::MovedPermanently()
                    .append_header((http::header::LOCATION, self.next_uri()))
                    .finish()
            }
            FlowStage::Authorized => {
                // 已授权，去拿 token
                HttpResponse::MovedPermanently()
                    .append_header((http::header::LOCATION, self.next_uri()))
                    .finish()
            }
            FlowStage::Completed => {
                // 已授权，去拿 token
                HttpResponse::Found()
                    .append_header((http::header::LOCATION, self.next_uri()))
                    .finish()
            }
        };
        resp.add_cookie(
            &Cookie::build("auth_session", &self.id)
                .max_age(Duration::minutes(10))
                .finish(),
        )?;
        Ok(resp)
    }
}

pub async fn validate_flow(req: &actix_web::HttpRequest) -> Result<Flow> {
    let session = req
        .cookie("auth_session")
        .ok_or(ApiError::ResponseError(ErrorPreconditionFailed(
            "auth_session is lacked",
        )))
        .map(|c| c.value().to_owned())?;

    redis_get::<Flow>(format!("forum:auth:flow:{}", session).as_str())
        .await
        .map_err(|_| ApiError::ResponseError(ErrorPreconditionFailed("session is nonexsistent")))?
        .ok_or(ApiError::ResponseError(ErrorPreconditionFailed(
            "session is expired",
        )))
        .and_then(|f| {
            if f.expires_at < Utc::now() {
                return Err(ApiError::ResponseError(ErrorPreconditionFailed(
                    "session is expired",
                )));
            }
            Ok(f)
        })
}

async fn persist_flow(flow: &'_ Flow) -> Result<&'_ Flow> {
    let now = Utc::now();
    if flow.expires_at < now {
        Err(ApiError::ResponseError(ErrorPreconditionFailed(
            "session is expired",
        )))
    } else {
        redis_setex(
            format!("forum:auth:flow:{}", flow.id).as_str(),
            flow,
            now - flow.expires_at,
        )
        .await?;
        Ok(flow)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ChallengeType {
    #[serde(rename = "email_code")]
    EmailCode,
    #[serde(rename = "email_link")]
    EmailLink,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ChallengeRequest {
    #[serde(rename = "client_id")]
    pub client_id: String,
    #[serde(rename = "type")]
    pub challenge_type: ChallengeType,
    #[serde(rename = "identifier")]
    pub identifier: String,
    #[serde(rename = "proof")]
    pub proof: String,
}
