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
    ws::{KucoinMultiSubscription, KucoinSubscription, KucoinWsEndpointResponse, KucoinWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::rest_api::NormalizedRestApiRequest,
    CexExchange
};

const BASE_REST_API_URL: &str = "https://api.kucoin.com";

#[derive(Debug, Default, Clone)]
pub struct Kucoin {
    subscriptions: Vec<KucoinSubscription>
}

impl Kucoin {
    pub fn new_ws_subscription(subscription: KucoinMultiSubscription) -> Self {
        Self { subscriptions: subscription.all_subscriptions() }
    }

    pub async fn get_websocket_endpoint() -> Result<KucoinWsEndpointResponse, WsError> {
        let data: KucoinWsEndpointResponse = reqwest::Client::new()
            .post(&format!("{BASE_REST_API_URL}/api/v1/bullet-public"))
            .send()
            .await
            .map_err(|e| WsError::WebInitializationError(e.to_string()))?
            .json()
            .await
            .map_err(|e| WsError::WebInitializationError(e.to_string()))?;

        Ok(data)
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
        let dyn_url = Self::get_websocket_endpoint().await?;

        let wss_endpoint = dyn_url
            .get_ws_endpoint()
            .ok_or(WsError::WebInitializationError("no websocket endpoints for Kucoin".to_string()))?;
        let wss_token = dyn_url.get_token();

        let wss_url = format!("{wss_endpoint}?token={wss_token}");
        let (mut ws, _) = tokio_tungstenite::connect_async(&wss_url).await?;

        for sub in self.subscriptions.iter() {
            let sub_message = serde_json::to_string(&sub)?;
            ws.send(Message::Text(sub_message)).await?;
        }

        Ok(ws)
    }

    async fn rest_api_call(
        &self,
        web_client: &reqwest::Client,
        api_channel: NormalizedRestApiRequest
    ) -> Result<KucoinRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => {
                KucoinRestApiResponse::Currencies(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/api/v3/currencies")).await?)
            }
            NormalizedRestApiRequest::AllInstruments => {
                KucoinRestApiResponse::Symbols(Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/api/v2/symbols")).await?)
            }
        };

        Ok(api_response)
    }
}
