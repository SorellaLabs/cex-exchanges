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
    rest_api::{OkexAllInstruments, OkexAllSymbols, OkexRestApiResponse},
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

    pub async fn get_all_symbols(&self, web_client: &reqwest::Client) -> Result<OkexAllSymbols, RestApiError> {
        let proxy_symbols = self.exch_currency_proxy.get_all_currencies().await?;
        let instruments = self.get_all_instruments(web_client).await?;

        Ok(OkexAllSymbols::new(proxy_symbols, instruments.instruments))
    }

    pub async fn get_all_instruments(&self, web_client: &reqwest::Client) -> Result<OkexAllInstruments, RestApiError> {
        let complete_instruments = join_all(NormalizedTradingType::iter().map(|t| async move {
            if t != NormalizedTradingType::Rfq {
                let instruments_with_type: OkexAllInstruments =
                    Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/api/v5/public/instruments?instType={t}")).await?;

                Ok(instruments_with_type)
            } else {
                Ok(OkexAllInstruments { instruments: vec![] })
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<OkexAllInstruments>, RestApiError>>()?
        .into_iter()
        .flat_map(|instr| instr.instruments)
        .collect::<Vec<_>>();

        Ok(OkexAllInstruments { instruments: complete_instruments })
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
impl Exchange for Okex {
    type RestApiResult = OkexRestApiResponse;
    type WsMessage = OkexWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Okex;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let url = if self.subscription.needs_business_ws() { WSS_BUSINESS_URL } else { WSS_PUBLIC_URL };

        let (mut ws, _) = tokio_tungstenite::connect_async(url).await?;

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
