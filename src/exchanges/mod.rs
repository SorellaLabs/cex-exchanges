pub mod normalized;

#[cfg(feature = "non-us")]
pub mod binance;

#[cfg(feature = "non-us")]
pub mod kucoin;

#[cfg(feature = "us")]
pub mod coinbase;
#[cfg(feature = "us")]
pub mod okex;

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use self::{
    binance::{ws::BinanceWsBuilder, Binance},
    coinbase::{ws::CoinbaseWsBuilder, Coinbase},
    kucoin::ws::KucoinWsBuilder,
    normalized::{
        rest_api::{NormalizedRestApiDataTypes, NormalizedRestApiRequest},
        types::NormalizedCurrency,
        ws::{CombinedWsMessage, NormalizedWsChannels}
    },
    okex::{ws::OkexWsBuilder, Okex}
};
use crate::{
    clients::{
        rest_api::RestApiError,
        ws::{MutliWsStream, WsError}
    },
    exchanges::normalized::rest_api::CombinedRestApiResponse
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, EnumIter)]
pub enum CexExchange {
    #[cfg(feature = "us")]
    Coinbase,
    #[cfg(feature = "us")]
    Okex,
    #[cfg(feature = "non-us")]
    Binance,
    #[cfg(feature = "non-us")]
    Kucoin
}

impl CexExchange {
    pub fn vec_all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn build_multistream_ws_from_normalized(
        &self,
        map: Vec<NormalizedWsChannels>,
        channels_per_stream: Option<usize>,
        split_channel_size: Option<usize>,
        exch_currency_proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStream> {
        let res = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => CoinbaseWsBuilder::make_from_normalized_map(map, split_channel_size)?
                .build_many_packed()?
                .build_multistream_unconnected(),
            #[cfg(feature = "us")]
            CexExchange::Okex => {
                OkexWsBuilder::make_from_normalized_map(map, channels_per_stream, exch_currency_proxy.unwrap_or(CexExchange::Binance))?
                    .build_many_packed()?
                    .build_multistream_unconnected()
            }
            #[cfg(feature = "non-us")]
            CexExchange::Binance => BinanceWsBuilder::make_from_normalized_map(map, channels_per_stream)?
                .build_many_packed()?
                .build_multistream_unconnected(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => KucoinWsBuilder::make_from_normalized_map(map, channels_per_stream)?
                .build_many_packed()?
                .build_multistream_unconnected()
        };

        Ok(res)
    }

    pub async fn get_all_currencies(&self) -> Result<Vec<NormalizedCurrency>, RestApiError> {
        let out = match self {
            #[cfg(feature = "us")]
            CexExchange::Coinbase => Coinbase::default()
                .rest_api_call(&reqwest::Client::new(), NormalizedRestApiRequest::AllCurrencies)
                .await?
                .normalize(),
            #[cfg(feature = "non-us")]
            CexExchange::Binance => Binance::default()
                .rest_api_call(&reqwest::Client::new(), NormalizedRestApiRequest::AllCurrencies)
                .await?
                .normalize(),
            #[cfg(feature = "us")]
            CexExchange::Okex => Okex::default()
                .rest_api_call(&reqwest::Client::new(), NormalizedRestApiRequest::AllCurrencies)
                .await?
                .normalize(),
            #[cfg(feature = "non-us")]
            CexExchange::Kucoin => Okex::default()
                .rest_api_call(&reqwest::Client::new(), NormalizedRestApiRequest::AllCurrencies)
                .await?
                .normalize()
        };

        match out {
            NormalizedRestApiDataTypes::AllCurrencies(vals) => Ok(vals),
            _ => unreachable!()
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
            CexExchange::Kucoin => write!(f, "kucoin")
        }
    }
}

#[async_trait::async_trait]
pub trait Exchange: Clone + Default {
    const EXCHANGE: CexExchange;
    type WsMessage: for<'de> Deserialize<'de> + Into<CombinedWsMessage> + Debug;
    type RestApiResult: for<'de> Deserialize<'de> + Into<CombinedRestApiResponse> + Debug;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>;

    async fn make_owned_ws_connection(self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        self.make_ws_connection().await
    }

    async fn rest_api_call(&self, web_client: &reqwest::Client, api_channel: NormalizedRestApiRequest) -> Result<Self::RestApiResult, RestApiError>;
}
