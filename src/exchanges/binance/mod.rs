mod pairs;

pub use pairs::*;

pub mod rest_api;
pub mod ws;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{BinanceAllInstruments, BinanceRestApiResponse, BinanceTradingDayTicker},
    ws::BinanceWsMessage
};
use crate::{
    binance::rest_api::BinanceAllInstrumentsUtil,
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

    pub async fn get_all_instruments(web_client: &reqwest::Client) -> Result<BinanceAllInstruments, RestApiError> {
        let trading_tickers: Vec<BinanceTradingDayTicker> =
            Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/ticker/24hr")).await?;

        let instruments: BinanceAllInstrumentsUtil = Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/exchangeInfo")).await?;

        Ok(BinanceAllInstruments::new(instruments.instruments, trading_tickers))
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
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => {
                BinanceRestApiResponse::Symbols(Self::simple_rest_api_request(web_client, ALL_SYMBOLS_URL.to_string()).await?)
            }
            NormalizedRestApiRequest::AllInstruments => BinanceRestApiResponse::Instruments(Self::get_all_instruments(web_client).await?)
        };

        Ok(api_response)
    }
}
