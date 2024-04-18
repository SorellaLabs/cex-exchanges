use super::{CoinbaseAllCurrenciesResponse, CoinbaseAllInstrumentsResponse};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseRestApiResponse {
    Currencies(CoinbaseAllCurrenciesResponse),
    Instruments(CoinbaseAllInstrumentsResponse)
}

impl CoinbaseRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            CoinbaseRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            CoinbaseRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_symbols(self) -> CoinbaseAllCurrenciesResponse {
        match self {
            CoinbaseRestApiResponse::Currencies(val) => val,
            _ => unreachable!()
        }
    }

    pub fn take_instruments(self) -> CoinbaseAllInstrumentsResponse {
        match self {
            CoinbaseRestApiResponse::Instruments(val) => val,
            _ => unreachable!()
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for CoinbaseRestApiResponse {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            CoinbaseRestApiResponse::Currencies(vals) => {
                matches!(normalized, NormalizedRestApiDataTypes::AllCurrencies(_)) && vals.equals_normalized()
            }
            CoinbaseRestApiResponse::Instruments(vals) => {
                matches!(normalized, NormalizedRestApiDataTypes::AllInstruments(_)) && vals.equals_normalized()
            }
        }
    }
}
