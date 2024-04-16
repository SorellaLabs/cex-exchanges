use thiserror::Error;

use crate::{exchanges::CexExchange, types::normalized::ws::combined::CombinedWsMessage};

#[derive(Debug, Error)]
pub enum WsError {
    #[error("failed to connect to the websocket: {0}")]
    ConnectionError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("failed to deserialize the message: {0}")]
    DeserializingError(#[from] serde_json::Error),
    #[error("recieved an error from the ws: {0}")]
    StreamRxError(tokio_tungstenite::tungstenite::Error),
    #[error("error sending value to the ws: {0}")]
    StreamTxError(tokio_tungstenite::tungstenite::Error),
    #[error("stream was terminated")]
    StreamTerminated
}

impl WsError {
    pub fn normalized_with_exchange(self, exchange: CexExchange) -> CombinedWsMessage {
        CombinedWsMessage::Disconnect { exchange, message: self.to_string() }
    }
}
