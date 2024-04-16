pub mod all_symbols;

use self::all_symbols::BinanceAllSymbolsResponse;
use crate::types::normalized::http::NormalizedHttpDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BinanceHttpResponse {
    Symbols(BinanceAllSymbolsResponse)
}

impl BinanceHttpResponse {
    pub fn normalize(self) -> NormalizedHttpDataTypes {
        match self {
            BinanceHttpResponse::Symbols(v) => NormalizedHttpDataTypes::AllCurrencies(v.normalize())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for BinanceHttpResponse {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            BinanceHttpResponse::Symbols(vals) => matches!(normalized, NormalizedHttpDataTypes::AllCurrencies(_)) && vals.equals_normalized()
        }
    }
}
