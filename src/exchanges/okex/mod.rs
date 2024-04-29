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

/*
    /// builds a mutlistream channel with a weighted mapping (how many channels
    /// to put per stream based on their 'exchange_ranking')
    ///
    /// [(#streams, #symbols/channel), ...]
    ///
    /// ex: [(2,3), (1,10), (1, 30), (1,55)]
    /// 2 streams with 3 symbols, 1 with 10 symbols, 1 with 20 symbols, 1
    /// with 55 symbols, 'n' streams with up to 1024 channels with the rest
    ///
    /// the default proxy exchange is [CexExchange::Binance]
    pub async fn build_all_weighted(
        weighted_map: Vec<(usize, usize)>,
        channels: &[OkexWsChannelKind],
        proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let proxy = proxy.unwrap_or(CexExchange::Binance);
        let this = Self::build_all_weighted_util(weighted_map, channels, proxy).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = OkexSubscription::new();
                subscription.add_channel(ch);

                Okex::new_ws_subscription(subscription, proxy)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_all_weighted_util(weighted_map: Vec<(usize, usize)>, channels: &[OkexWsChannelKind], proxy: CexExchange) -> eyre::Result<Self> {
        let mut this = Self::new(Some(proxy));

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Okex>()
            .await?
            .take_okex_instruments()
            .unwrap();

        all_symbols_vec.retain(|sy| sy.state == "live");

        // reverse sort by the sort order (low to high)

        let mut all_symbols = all_symbols_vec.into_iter();

        let mut map = weighted_map;
        map.sort_by(|a, b| b.1.cmp(&a.1));

        while let Some(nxt) = map.pop() {
            let (mut streams, num_channels) = nxt;
            while streams > 0 {
                let mut num_channels = num_channels;

                let mut symbols_chunk = Vec::new();
                while let Some(s) = all_symbols.next() {
                    symbols_chunk.push(s.instrument.try_into()?);
                    num_channels -= 1;
                    if num_channels == 0 {
                        break
                    }
                }

                let all_channels = channels
                    .iter()
                    .map(|ch| match ch {
                        OkexWsChannelKind::TradesAll => OkexWsChannel::TradesAll(symbols_chunk.clone()),
                        OkexWsChannelKind::BookTicker => OkexWsChannel::BookTicker(symbols_chunk.clone())
                    })
                    .collect::<Vec<_>>();

                this.channels.extend(all_channels);

                streams -= 1;
            }
        }

        let rest = all_symbols
            .map(|val| val.instrument.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let rest_stream_size = std::cmp::min(1024, rest.len());
        let rest_chunks = rest.chunks(rest_stream_size);

        rest_chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    OkexWsChannelKind::TradesAll => OkexWsChannel::TradesAll(chk.to_vec()),
                    OkexWsChannelKind::BookTicker => OkexWsChannel::BookTicker(chk.to_vec())
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        });

        Ok(this)
    }

*/
