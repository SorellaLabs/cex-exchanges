use super::NormalizedRestApiDataTypes;
#[cfg(feature = "non-us")]
use crate::exchanges::binance::rest_api::BinanceRestApiResponse;
#[cfg(feature = "us")]
use crate::exchanges::{coinbase::rest_api::CoinbaseRestApiResponse, okex::rest_api::OkexRestApiResponse};

#[derive(Debug, Clone)]
pub enum CombinedRestApiResponse {
    #[cfg(feature = "us")]
    Coinbase(CoinbaseRestApiResponse),
    #[cfg(feature = "us")]
    Okex(OkexRestApiResponse),
    #[cfg(feature = "non-us")]
    Binance(BinanceRestApiResponse)
}

impl CombinedRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Coinbase(c) => c.normalize(),
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Okex(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Binance(c) => c.normalize()
        }
    }
}

macro_rules! combined_rest {
    ($exchange:ident) => {
        paste::paste! {
            impl From<[<$exchange RestApiResponse>]> for CombinedRestApiResponse {
                fn from(value: [<$exchange RestApiResponse>]) -> Self {
                    Self::$exchange(value)
                }
            }
        }
    };
}

#[cfg(feature = "us")]
combined_rest!(Coinbase);

#[cfg(feature = "us")]
combined_rest!(Okex);

#[cfg(feature = "non-us")]
combined_rest!(Binance);

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for CombinedRestApiResponse {
    fn equals_normalized(self) -> bool {
        match self {
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Coinbase(vals) => vals.equals_normalized(),
            #[cfg(feature = "us")]
            CombinedRestApiResponse::Okex(vals) => vals.equals_normalized(),
            #[cfg(feature = "non-us")]
            CombinedRestApiResponse::Binance(vals) => vals.equals_normalized()
        }
    }
}
