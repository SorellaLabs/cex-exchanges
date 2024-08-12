pub mod normalized;

pub mod traits;

#[cfg(feature = "non-us")]
pub mod binance;

#[cfg(feature = "non-us")]
pub mod kucoin;

#[cfg(feature = "non-us")]
pub mod bybit;

#[cfg(feature = "us")]
pub mod coinbase;

#[cfg(feature = "us")]
pub mod okex;

use std::{
    fmt::{Debug, Display},
    str::FromStr
};

use clap::ValueEnum;
use futures::Future;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use traits::SpecificWsBuilder;

use self::normalized::{
    rest_api::NormalizedRestApiRequest,
    types::{NormalizedCurrency, NormalizedInstrument, NormalizedTradingPair},
    ws::{CombinedWsMessage, NormalizedWsChannels}
};
#[cfg(feature = "non-us")]
use self::{
    binance::{ws::BinanceWsBuilder, Binance, BinanceTradingPair},
    bybit::{ws::BybitWsBuilder, Bybit, BybitTradingPair},
    kucoin::{ws::KucoinWsBuilder, Kucoin, KucoinTradingPair}
};
#[cfg(feature = "us")]
use self::{
    coinbase::{ws::CoinbaseWsBuilder, Coinbase, CoinbaseTradingPair},
    okex::{ws::OkexWsBuilder, Okex, OkexTradingPair}
};
use crate::{
    clients::{
        rest_api::{ExchangeApi, RestApiError},
        ws::{CriticalWsMessage, MutliWsStream, WsError}
    },
    exchanges::normalized::rest_api::CombinedRestApiResponse,
    traits::ExchangeFilter
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, EnumIter, ValueEnum)]
pub enum CexExchange {
    #[cfg(feature = "us")]
    Coinbase,
    #[cfg(feature = "us")]
    Okex,
    #[cfg(feature = "non-us")]
    Binance,
    #[cfg(feature = "non-us")]
    Kucoin,
    #[cfg(feature = "non-us")]
    Bybit
}

impl CexExchange {
    pub fn vec_all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub(crate) fn build_multistream_ws_from_normalized(
        self,
        map: Vec<NormalizedWsChannels>,
        max_retries: Option<u64>,
        connections_per_stream: Option<usize>,
        exch_currency_proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStream> {
        let res = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => CoinbaseWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .build_multistream_unconnected(max_retries),
            #[cfg(feature = "us")]
            CexExchange::Okex => OkexWsBuilder::make_from_normalized_map(
                map,
                #[cfg(not(feature = "non-us"))]
                exch_currency_proxy.unwrap_or(CexExchange::Coinbase),
                #[cfg(feature = "non-us")]
                Some(exch_currency_proxy.unwrap_or(CexExchange::Binance))
            )?
            .build_many_packed(connections_per_stream)?
            .build_multistream_unconnected(max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => BinanceWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .build_multistream_unconnected(max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => KucoinWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .build_multistream_unconnected(max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => BybitWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .build_multistream_unconnected(max_retries)
        };

        Ok(res)
    }

    pub(crate) fn build_multithreaded_multistream_ws_from_normalized(
        self,
        map: Vec<NormalizedWsChannels>,
        exch_currency_proxy: Option<CexExchange>,
        max_retries: Option<u64>,
        connections_per_stream: Option<usize>,
        number_threads: usize
    ) -> eyre::Result<UnboundedReceiver<CombinedWsMessage>> {
        let res = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => CoinbaseWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .spawn_multithreaded(number_threads, max_retries),
            #[cfg(feature = "us")]
            CexExchange::Okex => OkexWsBuilder::make_from_normalized_map(
                map,
                #[cfg(not(feature = "non-us"))]
                exch_currency_proxy.unwrap_or(CexExchange::Coinbase),
                #[cfg(feature = "non-us")]
                Some(exch_currency_proxy.unwrap_or(CexExchange::Binance))
            )?
            .build_many_packed(connections_per_stream)?
            .spawn_multithreaded(number_threads, max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => BinanceWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .spawn_multithreaded(number_threads, max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => KucoinWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .spawn_multithreaded(number_threads, max_retries),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => BybitWsBuilder::make_from_normalized_map(map, None)?
                .build_many_packed(connections_per_stream)?
                .spawn_multithreaded(number_threads, max_retries)
        };

        Ok(res)
    }

    /// gets the normalized currencies from the different exchange endpoints
    /// ex: BNB, ETH, BTC
    ///
    /// if calling without a filter:
    /// ```
    /// CexExchange::Okex.get_all_currencies::<EmptyFilter>(None).await;
    /// ```
    pub async fn get_all_currencies<F>(self, filter: Option<F>) -> Result<Vec<NormalizedCurrency>, RestApiError>
    where
        F: ExchangeFilter<NormalizedCurrency>
    {
        let exchange_api = ExchangeApi::new();
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => exchange_api
                .all_currencies::<Coinbase>()
                .await?
                .normalize()
                .take_currencies(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => exchange_api
                .all_currencies::<Binance>()
                .await?
                .normalize()
                .take_currencies(filter)
                .unwrap(),
            #[cfg(feature = "us")]
            CexExchange::Okex => exchange_api
                .all_currencies::<Okex>()
                .await?
                .normalize()
                .take_currencies(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => exchange_api
                .all_currencies::<Kucoin>()
                .await?
                .normalize()
                .take_currencies(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => exchange_api
                .all_currencies::<Bybit>()
                .await?
                .normalize()
                .take_currencies(filter)
                .unwrap()
        };

        Ok(out)
    }

    /// gets the normalized instruments from the different exchange endpoints
    /// ex: BNB-ETH, ETHBTC, BTC/USDC
    ///
    /// if calling without a filter:
    /// ```
    /// CexExchange::Okex.get_all_instruments::<EmptyFilter>(None).await;
    /// ```
    pub async fn get_all_instruments<F>(self, filter: Option<F>) -> Result<Vec<NormalizedInstrument>, RestApiError>
    where
        F: ExchangeFilter<NormalizedInstrument>
    {
        let exchange_api = ExchangeApi::new();
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => exchange_api
                .all_instruments::<Coinbase>()
                .await?
                .normalize()
                .take_instruments(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => exchange_api
                .all_instruments::<Binance>()
                .await?
                .normalize()
                .take_instruments(filter)
                .unwrap(),
            #[cfg(feature = "us")]
            CexExchange::Okex => exchange_api
                .all_instruments::<Okex>()
                .await?
                .normalize()
                .take_instruments(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => exchange_api
                .all_instruments::<Kucoin>()
                .await?
                .normalize()
                .take_instruments(filter)
                .unwrap(),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => exchange_api
                .all_instruments::<Bybit>()
                .await?
                .normalize()
                .take_instruments(filter)
                .unwrap()
        };

        Ok(out)
    }

    /// converts a normalized trading pair back into the native exchange's pair
    pub fn denormalize_raw_trading_pair(self, pair: NormalizedTradingPair) -> eyre::Result<String> {
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => {
                let denorm_pair: CoinbaseTradingPair = pair.try_into()?;
                denorm_pair.0
            }
            #[cfg(feature = "non-us")]
            CexExchange::Binance => {
                let denorm_pair: BinanceTradingPair = pair.try_into()?;
                denorm_pair.0
            }
            #[cfg(feature = "us")]
            CexExchange::Okex => {
                let denorm_pair: OkexTradingPair = pair.try_into()?;
                denorm_pair.0
            }
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => {
                let denorm_pair: KucoinTradingPair = pair.try_into()?;
                denorm_pair.0
            }
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => {
                let denorm_pair: BybitTradingPair = pair.try_into()?;
                denorm_pair.0
            }
        };

        Ok(out)
    }

    #[allow(unreachable_patterns)]
    pub fn bad_pair(self, msg: String) -> Option<NormalizedTradingPair> {
        match self {
            CexExchange::Coinbase => CoinbaseTradingPair::parse_for_bad_pair(&msg).map(|p| p.normalize()),
            CexExchange::Okex => OkexTradingPair::parse_for_bad_pair(&msg).map(|p| p.normalize()),
            _ => None
        }
    }
}

impl Display for CexExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => write!(f, "coinbase"),
            #[cfg(feature = "us")]
            CexExchange::Okex => write!(f, "okex"),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => write!(f, "binance"),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => write!(f, "kucoin"),
            #[cfg(feature = "non-us")]
            CexExchange::Bybit => write!(f, "bybit")
        }
    }
}

impl FromStr for CexExchange {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.to_lowercase();

        match str.as_str() {
            #[cfg(feature = "us")]
            "coinbase" => Ok(CexExchange::Coinbase),
            #[cfg(feature = "us")]
            "okex" => Ok(CexExchange::Okex),
            #[cfg(feature = "non-us")]
            "binance" => Ok(CexExchange::Binance),
            #[cfg(feature = "non-us")]
            "kucoin" => Ok(CexExchange::Kucoin),
            #[cfg(feature = "non-us")]
            "bybit" => Ok(CexExchange::Bybit),
            _ => Err(eyre::ErrReport::msg(format!("'{s}' is not a valid exchange")))
        }
    }
}

pub trait Exchange: Clone + Default + Send {
    const EXCHANGE: CexExchange;
    type WsMessage: CriticalWsMessage + Send;
    type RestApiResult: for<'de> Deserialize<'de> + Into<CombinedRestApiResponse> + Debug + Send;

    fn remove_bad_pair(&mut self, bad_pair: NormalizedTradingPair) -> bool;

    fn make_ws_connection(&self) -> impl Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>> + Send;

    fn make_owned_ws_connection(self) -> impl Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>> + Send {
        async move { Box::pin(self.make_ws_connection()).await }
    }

    fn rest_api_call(
        &self,
        web_client: &reqwest::Client,
        api_channel: NormalizedRestApiRequest
    ) -> impl Future<Output = Result<Self::RestApiResult, RestApiError>> + Send;
}
