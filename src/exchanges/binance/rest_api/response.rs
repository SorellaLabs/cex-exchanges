use serde::{Deserialize, Serialize};

use super::BinanceAllSymbolsResponse;
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl PartialEq<NormalizedRestApiDataTypes> for BinanceRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            BinanceRestApiResponse::Symbols(vals) => vals == other // OkexRestApiResponse::Instruments(vals) => vals == other
        }
    }
}
