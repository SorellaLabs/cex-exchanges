use thiserror::Error;

use crate::{exchanges::normalized::ws::CombinedWsMessage, CexExchange};

#[derive(Debug, Error)]
pub enum WsError {
    #[error("failed to connect to the websocket: {0}")]
    ConnectionError(String),
    #[error("web initialization error: {0}")]
    WebInitializationError(String),
    #[error("failed to deserialize the message: {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("recieved an error from the ws: {0}")]
    StreamRxError(String),
    #[error("error sending value to the ws: {0}")]
    StreamTxError(String),
    #[error("stream was terminated")]
    StreamTerminated
}

impl WsError {
    pub fn normalized_with_exchange(self, exchange: CexExchange, raw_message: Option<String>) -> CombinedWsMessage {
        CombinedWsMessage::Disconnect { exchange, message: self.to_string(), raw_message: raw_message.unwrap_or(String::new()) }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for WsError {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::ConnectionError(value.to_string())
    }
}
