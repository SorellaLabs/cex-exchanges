use serde::{Deserialize, Serialize};

use super::{CoinbaseAllCurrenciesResponse, CoinbaseAllInstrumentsResponse};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseRestApiResponse {
    Currencies(CoinbaseAllCurrenciesResponse),
    Instruments(CoinbaseAllInstrumentsResponse)
}

impl CoinbaseRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            CoinbaseRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            CoinbaseRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_symbols(self) -> CoinbaseAllCurrenciesResponse {
        match self {
            CoinbaseRestApiResponse::Currencies(val) => val,
            _ => unreachable!()
        }
    }

    pub fn take_instruments(self) -> CoinbaseAllInstrumentsResponse {
        match self {
            CoinbaseRestApiResponse::Instruments(val) => val,
            _ => unreachable!()
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
