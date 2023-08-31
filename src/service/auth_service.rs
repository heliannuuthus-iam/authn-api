use actix_web::{HttpRequest, HttpResponse};
use http::header;

use super::idp::{
    github::GithubBuilder, google::Google, tencent::Tencent, wechat::WeChat, IdentifyProvider,
    IdpType,
};
use crate::{dto::auth::Flow, plugins::github::GitHubState};

// 生成认证链接
pub async fn authorize(flow: &Flow, requst: HttpRequest) -> HttpResponse {
    match flow.params.connection {
        IdpType::GitHub => {
            let github = GithubBuilder::default()
                .state(requst.app_data::<GitHubState>().unwrap().to_owned())
                .build()
                .unwrap();
            github.authentication(flow)
        }

        IdpType::Google => {
            let google = Google {};
            google.authentication(flow)
        }
        IdpType::WeChat => {
            let wechat: WeChat = WeChat {};
            wechat.authentication(flow)
        }
        IdpType::Tencent => {
            let tencent = Tencent {};
            tencent.authentication(flow)
        }
    };
    flow.next_uri()
}
