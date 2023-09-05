use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum SrpError {
    #[error("illegal_parameter: bad {0} value")]
    IllegalParameter(String),
    #[error("progress action error: {0}, msg: {1}")]
    ProgressError(&'static str, String),
    #[error("bad_record_mac: incorrect {0} proof")]
    BadRecordMac(String),
}
