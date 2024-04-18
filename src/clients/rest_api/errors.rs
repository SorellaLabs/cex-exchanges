use thiserror::Error;

#[derive(Debug, Error)]
pub enum RestApiError {
    #[error("failed to deserialize the message: {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("error sending request: {0}")]
    ReqwestError(#[from] reqwest::Error)
}
