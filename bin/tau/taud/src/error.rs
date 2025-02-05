use serde_json::Value;

use darkfi::rpc::jsonrpc::{ErrorCode, JsonError, JsonResponse, JsonResult};

#[derive(Debug, thiserror::Error)]
pub enum TaudError {
    #[error("Due timestamp invalid")]
    InvalidDueTime,
    #[error("Invalid Id")]
    InvalidId,
    #[error("Invalid Data/Params: `{0}` ")]
    InvalidData(String),
    #[error("InternalError")]
    Darkfi(#[from] darkfi::error::Error),
    #[error("Json serialization error: `{0}`")]
    SerdeJsonError(String),
    #[error("Encryption error: `{0}`")]
    EncryptionError(String),
    #[error("IO Error: `{0}`")]
    IoError(String),
}

pub type TaudResult<T> = std::result::Result<T, TaudError>;

impl From<serde_json::Error> for TaudError {
    fn from(err: serde_json::Error) -> TaudError {
        TaudError::SerdeJsonError(err.to_string())
    }
}

impl From<crypto_box::aead::Error> for TaudError {
    fn from(err: crypto_box::aead::Error) -> TaudError {
        TaudError::EncryptionError(err.to_string())
    }
}

impl From<std::io::Error> for TaudError {
    fn from(err: std::io::Error) -> TaudError {
        TaudError::IoError(err.to_string())
    }
}

pub fn to_json_result(res: TaudResult<Value>, id: Value) -> JsonResult {
    match res {
        Ok(v) => JsonResponse::new(v, id).into(),
        Err(err) => match err {
            TaudError::InvalidId => {
                JsonError::new(ErrorCode::InvalidParams, Some("invalid task id".into()), id).into()
            }
            TaudError::InvalidData(e) | TaudError::SerdeJsonError(e) => {
                JsonError::new(ErrorCode::InvalidParams, Some(e), id).into()
            }
            TaudError::InvalidDueTime => {
                JsonError::new(ErrorCode::InvalidParams, Some("invalid due time".into()), id).into()
            }
            TaudError::EncryptionError(e) => {
                JsonError::new(ErrorCode::InternalError, Some(e), id).into()
            }
            TaudError::Darkfi(e) => {
                JsonError::new(ErrorCode::InternalError, Some(e.to_string()), id).into()
            }
            TaudError::IoError(e) => JsonError::new(ErrorCode::InternalError, Some(e), id).into(),
        },
    }
}
