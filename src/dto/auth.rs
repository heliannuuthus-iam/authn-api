use thiserror::Error;

#[derive(Debug, Error)]
pub enum Erorr {
    #[error("{0}")]
    UnAuthentication(String),
    #[error("access_denied {0}")]
    AccessDenied(String),
}

pub struct Params {
    
}

pub struct Flow {
    params: Params,
    code: Option<u16>,
    message: Option<String>,
    error: Option<Erorr>,
}

impl Flow {
    pub fn new(params: Params) -> Flow {
        Flow {
            params: params,
            code: None,
            message: None,
            error: None,
        }
    }
}
