pub mod builder;
pub mod coinbase;
pub mod normalized;

use std::fmt::Debug;

use tokio::net::TcpStream;

use serde::Deserialize;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{types::normalized::ws::NormalizedWsMessage, ws::errors::WsError};

use self::normalized::CexExchange;

#[async_trait::async_trait]
pub trait Exchange: Clone {
    const EXCHANGE: CexExchange;
    type WsMessage: for<'de> Deserialize<'de> + Into<NormalizedWsMessage> + Debug;

    async fn make_ws_connection(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>;

    async fn make_owned_ws_connection(
        self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        self.make_ws_connection().await
    }
}
