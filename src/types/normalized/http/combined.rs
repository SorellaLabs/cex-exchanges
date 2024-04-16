use super::NormalizedHttpDataTypes;
use crate::types::coinbase::responses::CoinbaseHttpResponse;

#[derive(Debug, Clone)]
pub enum CombinedHttpResponse {
    Coinbase(CoinbaseHttpResponse)
}

impl CombinedHttpResponse {
    pub fn normalize(self) -> NormalizedHttpDataTypes {
        match self {
            CombinedHttpResponse::Coinbase(c) => c.normalize()
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

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CombinedHttpResponse {
    fn equals_normalized(self) -> bool {
        match self {
            CombinedHttpResponse::Coinbase(vals) => vals.equals_normalized()
        }
    }
}
