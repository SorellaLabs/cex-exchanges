use super::BinanceAllSymbolsResponse;
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BinanceRestApiResponse {
    Symbols(BinanceAllSymbolsResponse)
}

impl BinanceRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            BinanceRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for BinanceRestApiResponse {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            BinanceRestApiResponse::Symbols(vals) => matches!(normalized, NormalizedRestApiDataTypes::AllCurrencies(_)) && vals.equals_normalized()
        }
    }
}
