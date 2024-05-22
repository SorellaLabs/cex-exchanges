use serde::{Deserialize, Serialize};

use super::{CoinbaseAllCurrencies, CoinbaseAllProducts, CoinbaseCurrency, CoinbaseProduct};
use crate::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseRestApiResponse {
    Currencies(CoinbaseAllCurrencies),
    Products(CoinbaseAllProducts)
}

impl CoinbaseRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            CoinbaseRestApiResponse::Currencies(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            CoinbaseRestApiResponse::Products(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize())
        }
    }

    pub fn take_currencies(self) -> Option<Vec<CoinbaseCurrency>> {
        match self {
            CoinbaseRestApiResponse::Currencies(val) => Some(val.currencies),
            _ => None
        }
    }

    pub fn take_products(self, active_only: bool) -> Option<Vec<CoinbaseProduct>> {
        let instruments = match self {
            CoinbaseRestApiResponse::Products(val) => val.products,
            _ => return None
        };

        if active_only {
            Some(
                instruments
                    .into_iter()
                    .filter(|instr| !instr.trading_disabled)
                    .collect::<Vec<_>>()
            )
        } else {
            Some(instruments)
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CoinbaseRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            CoinbaseRestApiResponse::Currencies(vals) => vals == other,
            CoinbaseRestApiResponse::Products(vals) => vals == other
        }
    }
}
