use serde::{Deserialize, Serialize};

use super::{BybitAllCoins, BybitAllIntruments, BybitCoin, BybitIntrument};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BybitRestApiResponse {
    Coins(BybitAllCoins),
    Instruments(BybitAllIntruments)
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

    pub fn take_instruments(self) -> Option<Vec<BybitIntrument>> {
        match self {
            BybitRestApiResponse::Instruments(val) => Some(val.instruments),
            _ => None
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
