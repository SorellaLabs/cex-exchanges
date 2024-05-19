mod pairs;
use std::fmt::Debug;

pub use pairs::*;

pub mod rest_api;
pub mod ws;

use futures::SinkExt;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::CoinbaseRestApiResponse,
    ws::{CoinbaseSubscription, CoinbaseWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    normalized::{rest_api::NormalizedRestApiRequest, types::NormalizedTradingPair},
    CexExchange, Exchange
};

const WSS_URL: &str = "wss://ws-feed.exchange.coinbase.com";
const BASE_REST_API_URL: &str = "https://api.exchange.coinbase.com";

#[derive(Debug, Default, Clone)]
pub struct Coinbase {
    subscription: CoinbaseSubscription
}

impl Coinbase {
    pub fn new_ws_subscription(subscription: CoinbaseSubscription) -> Self {
        Self { subscription }
    }

    pub async fn simple_rest_api_request<T>(web_client: &reqwest::Client, url: String) -> Result<T, RestApiError>
    where
        T: for<'de> Deserialize<'de> + Debug
    {
        let data = web_client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("User-Agent", "rust")
            .header("Sec-WebSocket-Extensions", "permessage-deflate")
            .send()
            .await?
            .json()
            .await?;

        Ok(data)
    }
}

#[async_trait::async_trait]
impl Exchange for Coinbase {
    type RestApiResult = CoinbaseRestApiResponse;
    type WsMessage = CoinbaseWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Coinbase;

    fn remove_bad_pair(&mut self, bad_pair: NormalizedTradingPair) -> bool {
        let pair: CoinbaseTradingPair = bad_pair.try_into().unwrap();
        self.subscription.remove_pair(&pair)
    }

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (mut ws, _) = tokio_tungstenite::connect_async(WSS_URL).await?;

        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }

    async fn rest_api_call(
        &self,
        web_client: &reqwest::Client,
        api_channel: NormalizedRestApiRequest
    ) -> Result<CoinbaseRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => {
                CoinbaseRestApiResponse::Currencies(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/currencies")).await?)
            }
            NormalizedRestApiRequest::AllInstruments => {
                CoinbaseRestApiResponse::Products(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/products")).await?)
            }
        };

        Ok(api_response)
    }
}
