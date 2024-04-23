use super::{OkexAllInstruments, OkexAllSymbols};
use crate::normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedCurrency};

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum OkexRestApiResponse {
    Symbols(OkexAllSymbols),
    Instruments(OkexAllInstruments)
}

impl OkexRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            OkexRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            OkexRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<NormalizedCurrency>> {
        match self {
            OkexRestApiResponse::Symbols(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_instruments(self) -> Option<OkexAllInstruments> {
        match self {
            OkexRestApiResponse::Instruments(val) => Some(val),
            _ => None
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
