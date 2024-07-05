mod pairs;

use std::collections::{HashMap, HashSet};

use futures::SinkExt;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use reqwest::header;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};

use self::{
    rest_api::{BinanceAllInstruments, BinanceAllSymbols, BinanceRestApiResponse, BinanceSymbol},
    ws::{BinanceSubscription, BinanceWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::{rest_api::NormalizedRestApiRequest, types::NormalizedTradingPair},
    CexExchange
};

const WSS_URL: &str = "wss://stream.binance.com:443/stream";
const BASE_REST_API_URL: &str = "https://api.binance.com/api/v3";
const ALL_SYMBOLS_URL: &str = "https://www.binance.com/bapi/composite/v1/public/promo/cmc/cryptocurrency/listings/latest";

#[derive(Debug, Default, Clone)]
pub struct Binance {
    subscription: BinanceSubscription
}

impl Binance {
    pub fn new_ws_subscription(subscription: BinanceSubscription) -> Self {
        Self { subscription }
    }

    pub async fn get_all_instruments(web_client: &reqwest::Client) -> Result<BinanceAllInstruments, RestApiError> {
        let instruments: BinanceAllInstruments = Self::simple_rest_api_request(web_client, format!("{BASE_REST_API_URL}/exchangeInfo"), None).await?;
        info!(target: "cex-exchanges::binance", "found {} instruments", instruments.instruments.len());

        Ok(instruments)
    }

    pub async fn get_all_symbols(web_client: &reqwest::Client) -> Result<BinanceAllSymbols, RestApiError> {
        let instruments: BinanceAllInstruments = Self::get_all_instruments(web_client).await?;
        debug!(target: "cex-exchanges::binance", "got {} instruments to filter symbols", instruments.instruments.len());

        let pos_symbols = instruments
            .instruments
            .into_iter()
            .filter(|instr| &instr.status == "TRADING")
            .flat_map(|instr| vec![instr.base_asset, instr.quote_asset])
            .collect::<HashSet<_>>();

        debug!(target: "cex-exchanges::binance", "got {} symbols from instruments", pos_symbols.len());

        let mut query_start = 1;
        let mut symbols = HashMap::new();
        let mut err_count = 5;
        loop {
            debug!(target: "cex-exchanges::binance", "starting symbols iteration {query_start}");
            let symbols_iteration = match Self::symbols_iteration(web_client, query_start).await {
                Ok(vals) => {
                    if vals.is_empty() {
                        trace!(target: "cex-exchanges::binance", "no symbols found in valid call - breaking loop");
                        break
                    }
                    vals
                }
                Err(e) => {
                    if !e.is_gateway_timeout() {
                        err_count -= 1;
                        if err_count == 0 {
                            return Err(e)
                        }
                    }

                    warn!(target: "cex-exchanges::binance", "error getting symbols, {err_count} retries remaining - {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            symbols_iteration
                .into_iter()
                .filter(|sym| pos_symbols.contains(&sym.symbol))
                .for_each(|sym| {
                    symbols
                        .entry(sym.symbol.clone())
                        .and_modify(|curr_sym: &mut BinanceSymbol| {
                            if sym.cmc_rank < curr_sym.cmc_rank {
                                *curr_sym = sym.clone();
                            }
                        })
                        .or_insert(sym.clone());
                });

            query_start += 5000;
        }

        info!(target: "cex-exchanges::binance", "found {} valid symbols", symbols.values().len());

        Ok(BinanceAllSymbols { symbols: symbols.values().cloned().collect::<Vec<_>>() })
    }

    async fn symbols_iteration(web_client: &reqwest::Client, query_start: u64) -> Result<Vec<BinanceSymbol>, RestApiError> {
        let url = format!("{ALL_SYMBOLS_URL}?limit=5000&start={query_start}");
        let iter_symbols: BinanceAllSymbols =
            Self::simple_rest_api_request(web_client, url, Some((header::CONTENT_ENCODING, "gzip, deflate, br".parse().unwrap()))).await?;
        Ok(iter_symbols.symbols)
    }

    pub async fn simple_rest_api_request<T>(
        web_client: &reqwest::Client,
        url: String,
        extra_header: Option<(header::HeaderName, header::HeaderValue)>
    ) -> Result<T, RestApiError>
    where
        T: for<'de> Deserialize<'de>
    {
        let mut builder = web_client
            .get(&url)
            .header("Content-Type", "application/json");

        if let Some((k, v)) = extra_header {
            builder = builder.header(k, v);
        }

        let data = builder.send().await?.json().await?;

        Ok(data)
    }
}

impl Exchange for Binance {
    type RestApiResult = BinanceRestApiResponse;
    type WsMessage = BinanceWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Binance;

    fn remove_bad_pair(&mut self, bad_pair: NormalizedTradingPair) -> bool {
        let pair = bad_pair.try_into().unwrap();
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
    ) -> Result<BinanceRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => Self::get_all_symbols(web_client)
                .await
                .map(|v| BinanceRestApiResponse::Symbols(v)),
            NormalizedRestApiRequest::AllInstruments => Self::get_all_instruments(web_client)
                .await
                .map(|v| BinanceRestApiResponse::Instruments(v))
        };

        if let Err(e) = api_response.as_ref() {
            error!(target: "cex-exchanges::binance", "error calling rest-api endpoint {:?} -- {:?}", api_channel, e);
        }

        api_response
    }
}
