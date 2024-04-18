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

impl PartialEq<NormalizedRestApiDataTypes> for OkexRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            OkexRestApiResponse::Symbols(vals) => vals == other,
            OkexRestApiResponse::Instruments(vals) => vals == other
        }
    }
}
