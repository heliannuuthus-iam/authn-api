use http::{Method, StatusCode};
use reqwest::{Request, RequestBuilder, Response};
use tracing::info;

use crate::{
    common::{client::REQWEST, constant::FORUM_SERVER, errors::Result, nacos::rpc},
    dto::user::SubjectProfile,
};

pub async fn registry() -> Result<()> {
    RequestBuilder::from_parts(
        REQWEST.clone(),
        Request::new(
            Method::POST,
            rpc(format!("http://{FORUM_SERVER}/registry").as_str()).await?,
        ),
    )
    .send()
    .await
    .and_then(Response::error_for_status)?;
    Ok(())
}

pub async fn get_user_associations(openid: &str, idp: bool) -> Result<Option<SubjectProfile>> {
    info!("查询用户关联关系: openid({})", openid);
    let mut url = format!("http://{FORUM_SERVER}/users/associations/{openid}");
    if idp {
        url.push_str("/idp")
    }
    let resp = RequestBuilder::from_parts(
        REQWEST.clone(),
        Request::new(Method::GET, rpc(url.as_str()).await?),
    )
    .send()
    .await?;
    Ok(if StatusCode::NOT_FOUND.eq(&resp.status()) {
        info!("用户关联关系不存在 openid: {}", openid);
        None
    } else {
        Some(resp.error_for_status()?.json::<SubjectProfile>().await?)
    })
}
