use serde::Serialize;

use super::NormalizedRestApiDataTypes;
#[cfg(feature = "non-us")]
use crate::{
    binance::rest_api::{BinanceInstrument, BinanceRestApiResponse, BinanceSymbol},
    bybit::rest_api::{BybitCoin, BybitInstrument, BybitRestApiResponse},
    kucoin::rest_api::{KucoinCurrency, KucoinRestApiResponse, KucoinSymbol}
};
#[cfg(feature = "us")]
use crate::{
    coinbase::rest_api::{CoinbaseCurrency, CoinbaseProduct, CoinbaseRestApiResponse},
    okex::rest_api::{OkexCurrency, OkexInstrument, OkexRestApiResponse}
};

#[derive(Debug, Clone, Serialize)]
pub enum CombinedRestApiResponse {
    #[cfg(feature = "us")]
    Coinbase(CoinbaseRestApiResponse),
    #[cfg(feature = "us")]
    Okex(OkexRestApiResponse),
    #[cfg(feature = "non-us")]
    Binance(BinanceRestApiResponse),
    #[cfg(feature = "non-us")]
    Kucoin(KucoinRestApiResponse),
    #[cfg(feature = "non-us")]
    Bybit(BybitRestApiResponse)
}

impl CombinedRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Coinbase(c) => c.normalize(),
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Okex(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Binance(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Kucoin(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Bybit(c) => c.normalize()
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CombinedRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Coinbase(vals) => vals == other,
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Okex(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Binance(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Kucoin(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Bybit(vals) => vals == other
        }
    }
}

macro_rules! combined_exchange {
    ($exchange:ident, $currency:ident, $instrument:ident) => {
        paste::paste! {
            impl From<[<$exchange RestApiResponse>]> for CombinedRestApiResponse {
                fn from(value: [<$exchange RestApiResponse>]) -> Self {
                    Self::$exchange(value)
                }
            }

            impl CombinedRestApiResponse {
                pub fn [<take_ $exchange:lower _currencies>](self) -> Option<Vec<[<$exchange $currency>]>> {
                    self.[<take_ $exchange:lower>]().and_then(|v| v.[<take_ $currency:lower s>]())
                }

                pub fn [<take_ $exchange:lower _instruments>](self, active_only: bool) -> Option<Vec<[<$exchange $instrument>]>> {
                    self.[<take_ $exchange:lower>]().and_then(|v| v.[<take_ $instrument:lower s>](active_only))
                }

                pub fn[<take_ $exchange:lower>](self) -> Option<[<$exchange RestApiResponse>]> {
                    match self {
                        CombinedRestApiResponse::$exchange(val) => Some(val),
                        _ => None
                    }
                }
            }
        }
    };

    ($exchange:ident, (CURRENCY), $instrument:ident) => {
        paste::paste! {
            impl From<[<$exchange RestApiResponse>]> for CombinedRestApiResponse {
                fn from(value: [<$exchange RestApiResponse>]) -> Self {
                    Self::$exchange(value)
                }
            }

            impl CombinedRestApiResponse {
                pub fn [<take_ $exchange:lower _currencies>](self) -> Option<Vec<[<$exchange Currency>]>> {
                    self.[<take_ $exchange:lower>]().and_then(|v| v.take_currencies())
                }

                pub fn [<take_ $exchange:lower _instruments>](self, active_only: bool) -> Option<Vec<[<$exchange $instrument>]>> {
                    self.[<take_ $exchange:lower>]().and_then(|v| v.[<take_ $instrument:lower s>](active_only))
                }

                pub fn[<take_ $exchange:lower>](self) -> Option<[<$exchange RestApiResponse>]> {
                    match self {
                        CombinedRestApiResponse::$exchange(val) => Some(val),
                        _ => None
                    }
                }
            }
        }
    };
}

#[cfg(feature = "us")]
combined_exchange!(Coinbase, (CURRENCY), Product);

#[cfg(feature = "us")]
combined_exchange!(Okex, (CURRENCY), Instrument);

#[cfg(feature = "non-us")]
combined_exchange!(Kucoin, (CURRENCY), Symbol);

#[cfg(feature = "non-us")]
combined_exchange!(Binance, Symbol, Instrument);

#[cfg(feature = "non-us")]
combined_exchange!(Bybit, Coin, Instrument);
