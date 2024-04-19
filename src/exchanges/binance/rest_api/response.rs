use serde::{Deserialize, Serialize};

use super::{BinanceAllInstruments, BinanceAllSymbols, BinanceCompleteInstrument, BinanceSymbol};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BinanceRestApiResponse {
    Symbols(BinanceAllSymbols),
    Instruments(BinanceAllInstruments)
}

impl BinanceRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            BinanceRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            BinanceRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<BinanceSymbol>> {
        match self {
            BinanceRestApiResponse::Symbols(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_instruments(self) -> Option<Vec<BinanceCompleteInstrument>> {
        match self {
            BinanceRestApiResponse::Instruments(val) => Some(val.instruments),
            _ => None
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BinanceRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            BinanceRestApiResponse::Symbols(vals) => vals == other,
            BinanceRestApiResponse::Instruments(vals) => vals == other
        }
    }
}
