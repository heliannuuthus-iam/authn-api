use actix_web::HttpResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Erorr {
    #[error("unauthentication: {msg}")]
    UnAuthentication { code: http::StatusCode, msg: String },
    #[error("access_denied msg")]
    AccessDenied { code: http::StatusCode, msg: String },
}

pub struct Params {
    connection: String,
}

pub struct Flow {
    params: Params,
    code: http::StatusCode,
    error: Option<Erorr>,
    message: Option<String>,
}

impl Flow {
    pub fn new(params: Params) -> Flow {
        Flow {
            params: params,
            code: StatusCode::OK,
            message: None,
            error: None,
        }
    }

    pub fn misinterpret(&self) -> &Self {
        match self.code {
            StatusCode::UNAUTHORIZED => {}

            _ => {
                if (self.code.is_server_error()) {
                    self.error = Erorr::UnAuthentication { code: (Http), msg: () }
                }
            }
        }

        self
    }
}
