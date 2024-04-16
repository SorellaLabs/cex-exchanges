#[cfg(feature = "non-us")]
pub mod binance;
pub mod builder;
pub mod coinbase;

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{
    clients::{http::errors::HttpError, ws::errors::WsError},
    types::normalized::{http::combined::CombinedHttpResponse, ws::combined::CombinedWsMessage}
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CexExchange {
    Coinbase,
    Binance
}

impl Display for CexExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CexExchange::Coinbase => write!(f, "coinbase"),
            CexExchange::Binance => write!(f, "binance")
        }
    }
}

#[async_trait::async_trait]
pub trait Exchange: Clone {
    const EXCHANGE: CexExchange;
    type WsMessage: for<'de> Deserialize<'de> + Into<CombinedWsMessage> + Debug;
    type HttpMessage: for<'de> Deserialize<'de> + Into<CombinedHttpResponse> + Debug;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>;

    async fn make_owned_ws_connection(self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        self.make_ws_connection().await
    }

    async fn all_symbols(web_client: &reqwest::Client) -> Result<Self::HttpMessage, HttpError>;
}
