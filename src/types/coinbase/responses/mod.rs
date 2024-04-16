use self::all_currencies::CoinbaseAllCurrenciesResponse;
use crate::types::normalized::http::NormalizedHttpDataTypes;

pub mod all_currencies;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseHttpResponse {
    Currencies(CoinbaseAllCurrenciesResponse)
}

impl CoinbaseHttpResponse {
    pub fn normalize(self) -> NormalizedHttpDataTypes {
        match self {
            CoinbaseHttpResponse::Currencies(v) => NormalizedHttpDataTypes::AllCurrencies(v.normalize())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseHttpResponse {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            CoinbaseHttpResponse::Currencies(vals) => matches!(normalized, NormalizedHttpDataTypes::AllCurrencies(_)) && vals.equals_normalized()
        }
    }
}
