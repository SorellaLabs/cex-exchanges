mod pairs;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{BinanceAllSymbolsResponse, BinanceRestApiResponse},
    ws::BinanceWsMessage
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::rest_api::NormalizedRestApiRequest,
    CexExchange
};

const WSS_URL: &str = "wss://stream.binance.com:443/stream?streams=";
const BASE_REST_API_URL: &str = "https://api.binance.com/api/v3";
const ALL_SYMBOLS_URL: &str = "https://www.binance.com/bapi/composite/v1/public/promo/cmc/cryptocurrency/listings/latest";

#[derive(Debug, Default, Clone)]
pub struct Binance {
    ws_url: String
}

impl Binance {
    pub fn new_ws_subscription(ws_url: String) -> Self {
        Self { ws_url }
    }
}

#[async_trait::async_trait]
impl Exchange for Binance {
    type RestApiMessage = BinanceRestApiResponse;
    type WsMessage = BinanceWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Binance;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (ws, _) = tokio_tungstenite::connect_async(&self.ws_url).await?;

        Ok(ws)
    }

    async fn rest_api_call(
        &self,
        web_client: &reqwest::Client,
        api_channel: NormalizedRestApiRequest
    ) -> Result<BinanceRestApiResponse, RestApiError> {
        let currencies: BinanceAllSymbolsResponse = web_client.get(ALL_SYMBOLS_URL).send().await?.json().await?;

        Ok(BinanceRestApiResponse::Symbols(currencies))
    }
}
