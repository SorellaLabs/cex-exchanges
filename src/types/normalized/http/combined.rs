use super::NormalizedHttpDataTypes;
#[cfg(feature = "non-us")]
use crate::types::binance::responses::BinanceHttpResponse;
use crate::types::coinbase::responses::CoinbaseHttpResponse;

#[derive(Debug, Clone)]
pub enum CombinedHttpResponse {
    Coinbase(CoinbaseHttpResponse),
    #[cfg(feature = "non-us")]
    Binance(crate::types::binance::responses::BinanceHttpResponse)
}

impl CombinedHttpResponse {
    pub fn normalize(self) -> NormalizedHttpDataTypes {
        match self {
            CombinedHttpResponse::Coinbase(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedHttpResponse::Binance(c) => c.normalize()
        }
    }
}

macro_rules! combined_http {
    ($exchange:ident) => {
        paste::paste! {
            impl From<[<$exchange HttpResponse>]> for CombinedHttpResponse {
                fn from(value: [<$exchange HttpResponse>]) -> Self {
                    Self::$exchange(value)
                }
            }
        }
    };
}

combined_http!(Coinbase);
#[cfg(feature = "non-us")]
combined_http!(Binance);

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CombinedHttpResponse {
    fn equals_normalized(self) -> bool {
        match self {
            CombinedHttpResponse::Coinbase(vals) => vals.equals_normalized(),
            #[cfg(feature = "non-us")]
            CombinedHttpResponse::Binance(vals) => vals.equals_normalized()
        }
    }
}
