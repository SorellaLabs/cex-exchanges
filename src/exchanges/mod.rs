pub mod builder;
pub mod coinbase;
pub mod normalized;

use std::fmt::Debug;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::normalized::CexExchange;
use crate::{
    http::errors::HttpError,
    types::normalized::{http::combined::CombinedHttpResponse, ws::combined::CombinedWsMessage},
    ws::errors::WsError
};

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
