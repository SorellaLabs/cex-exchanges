use serde::{Deserialize, Serialize};

use super::{KucoinAllCurrencies, KucoinAllSymbols, KucoinCurrency, KucoinSymbol};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KucoinRestApiResponse {
    Currencies(KucoinAllCurrencies),
    Symbols(KucoinAllSymbols)
}

impl KucoinRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            KucoinRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            KucoinRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<KucoinCurrency>> {
        match self {
            KucoinRestApiResponse::Currencies(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_symbols(self) -> Option<Vec<KucoinSymbol>> {
        match self {
            KucoinRestApiResponse::Symbols(val) => Some(val.symbols),
            _ => None
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for KucoinRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            KucoinRestApiResponse::Currencies(vals) => vals == other,
            KucoinRestApiResponse::Symbols(vals) => vals == other
        }
    }
}
