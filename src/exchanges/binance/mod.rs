mod pairs;
use futures::StreamExt;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{BinanceAllInstruments, BinanceInstrument, BinanceRestApiResponse, BinanceTradingDayTicker},
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

    pub(crate) async fn get_all_instruments_util(web_client: &reqwest::Client) -> Result<Vec<BinanceInstrument>, RestApiError> {
        let val: BinanceAllInstrumentsUtil = Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/exchangeInfo")).await?;
        Ok(val.instruments)
    }

    pub async fn get_all_instruments(web_client: &reqwest::Client) -> Result<BinanceAllInstruments, RestApiError> {
        let instruments: Vec<BinanceInstrument> = Self::get_all_instruments_util(web_client).await?;

        // build_url_extension_from_symbols
        let symbols_url = &format!("{BASE_REST_API_URL}/ticker/tradingDay?symbols=");
        let symbol_chunks = BinanceTradingDayTicker::build_url_extension_from_symbols(&instruments);
        let num_chunks = symbol_chunks.len();

        let mut trading_tickers = Vec::new();
        let mut trading_tickers_stream = futures::stream::iter(symbol_chunks)
            .map(|chk| async move {
                let inner_url = format!("{symbols_url}{chk}");
                let out: Vec<BinanceTradingDayTicker> = Self::simple_rest_api_request(web_client, inner_url).await?;
                Ok(out) as Result<Vec<BinanceTradingDayTicker>, RestApiError>
            })
            .buffer_unordered(1);

        // 6000 weighted request/min
        // 4 weight/symbol
        // 50 symbols/rquest
        // 2 requests/sec
        let mut i = 0;
        while let Some(tt) = trading_tickers_stream.next().await {
            i += 1;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            trading_tickers.extend(tt.unwrap());
            println!("Completed Binance symbols chunk {}/{num_chunks}", i);
        }

        Ok(BinanceAllInstruments::new(instruments, trading_tickers))
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
