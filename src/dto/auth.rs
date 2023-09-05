use actix_web::{
    cookie::{time::Duration, Cookie},
    HttpResponse,
};
use chrono::{DateTime, Utc};
use http::Uri;
use tracing::error;
use validator::Validate;

use crate::{
    common::{
        cache::{cache_get, cache_setex},
        errors::{ApiError, Result},
        oauth::{AuthCodeResponse, IdpType, OAuthUser},
        utils::gen_id,
    },
    dto::user::{UserAssociation, UserProfile},
};

#[derive(Debug, Clone, thiserror::Error, serde::Deserialize, serde::Serialize)]
pub enum AuthError {
    #[error("un_authenticate: {0}")]
    UnAuthenticate(String),
    #[error("access_denied: {0}")]
    AccessDenied(String),
    #[error("user not found: {0}")]
    NotFound(String),
    #[error("unprocessable entity: {0}")]
    UnprocessableContent(String),
    #[error("auth header error: {0}")]
    PreconditionFailed(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Validate, Default)]
pub struct Params {
    #[validate(length(min = 21, max = 22))]
    pub client_id: String,
    pub connection: IdpType,
    #[validate(url)]
    pub redirect_uri: String,
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
    pub params: Params,
    pub code_resp: Option<AuthCodeResponse>,
    pub redirect_url: Option<String>,
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

    pub fn new(params: Params) -> Self {
        Flow {
            id: Self::gen_id(),
            params,
            ..Default::default()
        }
    }

    pub fn next_uri(&self) -> String {
        let mut builder = Uri::builder().scheme("http");

        builder = match self.transfer_error() {
            Some(auth_err) => builder
                .path_and_query("/done")
                .path_and_query(format!("error={auth_err}")),
            None => builder,
        };

        let next_uri = match self.stage {
            FlowStage::Initialized => "/login",
            FlowStage::Authenticating => "/login",
            FlowStage::Authenticated => "/confirm",
            FlowStage::Authorized => "/done",
            FlowStage::Completed => "/done",
        };
        builder
            .path_and_query(next_uri)
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

    fn transfer_error<'a>(&self) -> Option<&'a str> {
        if let Some(ref err) = self.error {
            match err {
                AuthError::UnAuthenticate(_msg) => Some("invalid_client"),
                AuthError::AccessDenied(_msg) => Some("access_denied"),
                AuthError::UnprocessableContent(_msg) => Some("unproccessed"),
                _ => Some("unkonw"),
            }
        } else {
            None
        }
    }
}

pub async fn validate_flow(req: &actix_web::HttpRequest) -> Result<Flow> {
    let session = req
        .cookie("auth_session")
        .ok_or(ApiError::PreconditionFailed(format!(
            "auth_session is lacked"
        )))
        .map(|c| c.value().to_owned())?;

    cache_get::<Flow>(format!("forum:auth:flow:{}", session).as_str())
        .await
        .map_err(|_| ApiError::PreconditionFailed("session is nonexsistent".to_string()))?
        .ok_or(ApiError::PreconditionFailed(
            "session is expired".to_string(),
        ))
        .and_then(|f| {
            if f.expires_at < Utc::now() {
                return Err(ApiError::PreconditionFailed(
                    "session is expired".to_string(),
                ));
            }
            Ok(f)
        })
}

async fn persist_flow(flow: &'_ Flow) -> Result<&'_ Flow> {
    let now = Utc::now();
    if flow.expires_at < now {
        Err(ApiError::PreconditionFailed(
            "session is expired".to_string(),
        ))?
    } else {
        cache_setex(
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
    client_id: String,
    #[serde(rename = "type")]
    ctype: ChallengeType,
    #[serde(rename = "identifier")]
    cer: String,
    #[serde(rename = "proof")]
    proof: String,
}
