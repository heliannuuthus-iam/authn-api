use super::idp::{
    github::GithubBuilder, google::Google, tencent::Tencent, wechat::WeChat, IdentifyProvider,
    IdpType,
};
use crate::plugins::github::GitHubState;
use actix_web::{HttpRequest, HttpResponse};
use http::header;

// 生成认证链接
pub async fn authorize(idp_type: IdpType, requst: HttpRequest) -> HttpResponse {
    let result = match idp_type {
        IdpType::GitHub => {
            let github = GithubBuilder::default()
                .state(requst.app_data::<GitHubState>().unwrap().to_owned())
                .build()
                .unwrap();
            github.login()
        }

        IdpType::Google => {
            let google = Google {};
            google.login()
        }
        IdpType::WeChat => {
            let wechat = WeChat {};
            wechat.login()
        }
        IdpType::Tencent => {
            let tencent = Tencent {};
            tencent.login()
        }
    };
    HttpResponse::Found()
        .append_header((header::LOCATION, result.to_string()))
        .finish()
}
