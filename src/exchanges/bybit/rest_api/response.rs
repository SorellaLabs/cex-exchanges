use serde::{Deserialize, Serialize};

use super::{BybitAllCoins, BybitAllInstruments, BybitCoin, BybitInstrument};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BybitRestApiResponse {
    Coins(BybitAllCoins),
    Instruments(BybitAllInstruments)
}

impl BybitRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            BybitRestApiResponse::Coins(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            BybitRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_coins(self) -> Option<Vec<BybitCoin>> {
        match self {
            BybitRestApiResponse::Coins(val) => Some(val.coins),
            _ => None
        }
    }

    pub fn take_instruments(self, active_only: bool) -> Option<Vec<BybitInstrument>> {
        let instruments = match self {
            BybitRestApiResponse::Instruments(val) => val.instruments,
            _ => return None
        };

        if active_only {
            Some(
                instruments
                    .into_iter()
                    .filter(|instr| instr.inner.status == "Trading")
                    .collect::<Vec<_>>()
            )
        } else {
            Some(instruments)
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            BybitRestApiResponse::Coins(vals) => vals == other,
            BybitRestApiResponse::Instruments(vals) => vals == other
        }
    }
}
