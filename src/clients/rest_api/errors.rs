use thiserror::Error;

#[derive(Debug, Error)]
pub enum RestApiError {
    #[error("failed to deserialize the message: {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("error sending request: {0}")]
    ReqwestError(#[from] reqwest::Error)
}

impl RestApiError {
    pub(crate) fn is_gateway_timeout(&self) -> bool {
        match self {
            Self::ReqwestError(err) => {
                if let Some(code) = err.status() {
                    return code.as_u16() == 504
                }
            }
            _ => ()
        }
        false
    }
}
