use super::{OkexAllSymbolsResponse, OkexCompleteAllInstruments};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum OkexRestApiResponse {
    Symbols(OkexAllSymbolsResponse),
    Instruments(OkexCompleteAllInstruments)
}

impl OkexRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            OkexRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            OkexRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for OkexRestApiResponse {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            OkexRestApiResponse::Symbols(vals) => matches!(normalized, NormalizedRestApiDataTypes::AllCurrencies(_)) && vals.equals_normalized(),
            OkexRestApiResponse::Instruments(vals) => matches!(normalized, NormalizedRestApiDataTypes::AllInstruments(_)) && vals.equals_normalized()
        }
    }
}
