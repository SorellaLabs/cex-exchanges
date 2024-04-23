use serde::{Deserialize, Serialize};

use super::{BybitAllCurrencies, BybitAllSymbols, BybitCurrency, BybitSymbol};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BybitRestApiResponse {
    Currencies(BybitAllCurrencies),
    Symbols(BybitAllSymbols)
}

impl BybitRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            BybitRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            BybitRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<BybitCurrency>> {
        match self {
            BybitRestApiResponse::Currencies(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_symbols(self) -> Option<Vec<BybitSymbol>> {
        match self {
            BybitRestApiResponse::Symbols(val) => Some(val.symbols),
            _ => None
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            BybitRestApiResponse::Currencies(vals) => vals == other,
            BybitRestApiResponse::Symbols(vals) => vals == other
        }
    }
}
