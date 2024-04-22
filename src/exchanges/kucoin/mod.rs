mod pairs;

use futures::SinkExt;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::KucoinRestApiResponse,
    ws::{KucoinSubscription, KucoinWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::rest_api::NormalizedRestApiRequest,
    CexExchange
};

const WSS_URL: &str = "wss://stream.kucoin.com:443/stream";
const BASE_REST_API_URL: &str = "https://api.kucoin.com";

#[derive(Debug, Default, Clone)]
pub struct Kucoin {
    subscription: KucoinSubscription
}

impl Kucoin {
    pub fn new_ws_subscription(subscription: KucoinSubscription) -> Self {
        Self { subscription }
    }

    pub async fn simple_rest_api_request<T>(web_client: &reqwest::Client, url: String) -> Result<T, RestApiError>
    where
        T: for<'de> Deserialize<'de>
    {
        let data = web_client.get(&url).send().await?.json().await?;
        Ok(data)
    }
}

#[async_trait::async_trait]
impl Exchange for Kucoin {
    type RestApiResult = KucoinRestApiResponse;
    type WsMessage = KucoinWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Kucoin;

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
    ) -> Result<KucoinRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => {
                KucoinRestApiResponse::Currencies(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/v3/currencies")).await?)
            }
            NormalizedRestApiRequest::AllInstruments => {
                KucoinRestApiResponse::Symbols(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/v2/symbols")).await?)
            }
        };

        Ok(api_response)
    }
}
