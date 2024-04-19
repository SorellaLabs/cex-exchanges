use serde::{Deserialize, Serialize};

use super::{CoinbaseAllCurrencies, CoinbaseAllInstruments, CoinbaseCompleteInstrument, CoinbaseCurrency};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseRestApiResponse {
    Currencies(CoinbaseAllCurrencies),
    Instruments(CoinbaseAllInstruments)
}

impl CoinbaseRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            CoinbaseRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            CoinbaseRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<CoinbaseCurrency>> {
        match self {
            CoinbaseRestApiResponse::Currencies(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_instruments(self) -> Option<Vec<CoinbaseCompleteInstrument>> {
        match self {
            CoinbaseRestApiResponse::Instruments(val) => Some(val.instruments),
            _ => None
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CoinbaseRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            CoinbaseRestApiResponse::Currencies(vals) => vals == other,
            CoinbaseRestApiResponse::Instruments(vals) => vals == other
        }
    }
}
