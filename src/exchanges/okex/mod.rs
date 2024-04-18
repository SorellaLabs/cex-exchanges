mod pairs;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use futures::{future::join_all, SinkExt};
use serde::Deserialize;
use strum::IntoEnumIterator;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{OkexAllInstrumentsResponse, OkexAllSymbolsResponse, OkexAllTickersResponse, OkexCompleteAllInstruments, OkexRestApiResponse},
    ws::{OkexSubscription, OkexWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    normalized::{rest_api::NormalizedRestApiRequest, types::NormalizedTradingType},
    CexExchange, Exchange
};

const WSS_PUBLIC_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
const WSS_BUSINESS_URL: &str = "wss://ws.okx.com:8443/ws/v5/business";
const BASE_REST_API_URL: &str = "https://www.okx.com";

#[derive(Debug, Clone)]
pub struct Okex {
    subscription:        OkexSubscription,
    /// exchange to use to get the symbols (since there is no direct symbols
    /// api) - default is binance
    exch_currency_proxy: CexExchange
}

impl Okex {
    pub fn new_ws_subscription(subscription: OkexSubscription, exch_currency_proxy: CexExchange) -> Self {
        Self { subscription, exch_currency_proxy }
    }

    pub async fn get_all_symbols(&self, web_client: &reqwest::Client) -> Result<OkexAllSymbolsResponse, RestApiError> {
        let proxy_symbols = self.exch_currency_proxy.get_all_currencies().await?;

        let currencies: OkexAllSymbolsResponse = web_client
            .get(BASE_REST_API_URL)
            .send()
            .await?
            .json()
            .await?;

        Ok(currencies)
    }

    pub async fn get_all_instruments(&self, web_client: &reqwest::Client) -> Result<OkexCompleteAllInstruments, RestApiError> {
        use std::io::Write;
        let tt = [NormalizedTradingType::Margin];

        let complete_instruments = join_all(tt.iter().map(|t| async move {
            let tickers: OkexAllTickersResponse = if *t == NormalizedTradingType::Margin {
                Self::simple_rest_api_request(
                    web_client,
                    format!("{BASE_REST_API_URL}/api/v5/market/tickers?instType={}", NormalizedTradingType::Spot.fmt_okex())
                )
                .await?
            } else {
                Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/api/v5/market/tickers?instType={}", t.fmt_okex())).await?
            };

            let mut f0 = std::fs::File::create("/Users/josephnoorchashm/Desktop/SorellaLabs/GitHub/cex-exchanges/t0.json").unwrap();
            writeln!(f0, "{}", serde_json::to_string(&tickers).unwrap()).unwrap();

            let instruments_no_vol: OkexAllInstrumentsResponse =
                Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/api/v5/public/instruments?instType={}", t.fmt_okex())).await?;

            let mut f1 = std::fs::File::create("/Users/josephnoorchashm/Desktop/SorellaLabs/GitHub/cex-exchanges/t1.json").unwrap();
            writeln!(f1, "{}", serde_json::to_string(&instruments_no_vol).unwrap()).unwrap();

            Ok((tickers, instruments_no_vol).into())
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<OkexCompleteAllInstruments>, RestApiError>>()?
        .into_iter()
        .flat_map(|instr| instr.instruments)
        .collect::<Vec<_>>();

        Ok(OkexCompleteAllInstruments { instruments: complete_instruments })
    }

    pub async fn simple_rest_api_request<T>(web_client: &reqwest::Client, url: String) -> Result<T, RestApiError>
    where
        T: for<'de> Deserialize<'de>
    {
        let data = web_client.get(&url).send().await?.text().await?;

        let res = serde_json::from_str(&data);

        if res.is_err() {
            println!("\n\nURL: {url}\nDATA: {data}\n\n");
        }

        Ok(res?)
    }
}

#[async_trait::async_trait]
impl Exchange for Okex {
    type RestApiMessage = OkexRestApiResponse;
    type WsMessage = OkexWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Okex;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let url = if self.subscription.needs_business_ws() { WSS_BUSINESS_URL } else { WSS_PUBLIC_URL };

        let (mut ws, _) = tokio_tungstenite::connect_async(url).await?;

        //        println!("SUB: {:?}", self.subscription);
        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }

    async fn rest_api_call(&self, web_client: &reqwest::Client, api_channel: NormalizedRestApiRequest) -> Result<OkexRestApiResponse, RestApiError> {
        let call_result = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => OkexRestApiResponse::Symbols(self.get_all_symbols(web_client).await?),
            NormalizedRestApiRequest::AllInstruments => OkexRestApiResponse::Instruments(self.get_all_instruments(web_client).await?)
        };

        Ok(call_result)
    }
}

impl Default for Okex {
    fn default() -> Self {
        Self { subscription: Default::default(), exch_currency_proxy: CexExchange::Binance }
    }
}
