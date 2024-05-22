use super::{OkexAllInstruments, OkexAllSymbols, OkexCurrency, OkexInstrument};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

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

    pub fn take_currencies(self) -> Option<Vec<OkexCurrency>> {
        match self {
            OkexRestApiResponse::Symbols(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_instruments(self, active_only: bool) -> Option<Vec<OkexInstrument>> {
        let instruments = match self {
            OkexRestApiResponse::Instruments(val) => val.instruments,
            _ => return None
        };

        if active_only {
            Some(
                instruments
                    .into_iter()
                    .filter(|instr| &instr.state.to_lowercase() == "live")
                    .collect()
            )
        } else {
            None
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
