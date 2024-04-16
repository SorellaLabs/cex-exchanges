use crate::types::coinbase::channels::CoinbaseSubscription;
use crate::types::coinbase::messages::CoinbaseWsMessage;

use super::normalized::CexExchange;
use super::Exchange;
use futures::SinkExt;

use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::types::coinbase::channels::CoinbaseChannel;
use crate::ws::errors::WsError;

const WSS_URL: &str = "wss://ws-feed.exchange.coinbase.com";

#[derive(Debug, Clone)]
pub struct Coinbase {
    pub url: &'static str,
    subscription: CoinbaseSubscription,
}

impl Coinbase {
    pub fn new_with_subscription(sub: CoinbaseSubscription) -> Self {
        Self {
            url: WSS_URL,
            subscription: sub,
        }
    }
}

#[async_trait::async_trait]
impl Exchange for Coinbase {
    const EXCHANGE: CexExchange = CexExchange::Coinbase;
    type WsMessage = CoinbaseWsMessage;

    async fn make_ws_connection(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (mut ws, _) = tokio_tungstenite::connect_async(&*self.url).await?;

        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CoinbaseWsBuilder {
    pub channels: Vec<CoinbaseChannel>,
}

impl CoinbaseWsBuilder {
    pub fn add_channel(mut self, channel: CoinbaseChannel) -> Self {
        self.channels.push(channel);
        self
    }

    pub fn build(self) -> Coinbase {
        let mut sub = CoinbaseSubscription::new();

        self.channels
            .into_iter()
            .for_each(|c| sub.add_channel(c.into()));

        Coinbase::new_with_subscription(sub)
    }
}
