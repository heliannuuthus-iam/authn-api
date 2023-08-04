use crate::plugins::client::{Error, WEB_CLIENT};
use oauth2::{HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct  AuthRequest {
    pub code: String,
    pub state: String,
}



pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, Error<reqwest::Error>> {
    let mut request_builder = WEB_CLIENT
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::Reqwest)?;

    let response = WEB_CLIENT.execute(request).await.map_err(Error::Reqwest)?;

    Ok(HttpResponse {
        status_code: response.status(),
        headers: response.headers().to_owned(),
        body: response.bytes().await.map_err(Error::Reqwest)?.to_vec(),
    })
}
