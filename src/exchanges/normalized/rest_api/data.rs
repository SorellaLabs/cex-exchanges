use serde::Serialize;

use crate::normalized::types::{NormalizedCurrency, NormalizedInstrument};

#[derive(Debug, Clone, Serialize)]
pub enum NormalizedRestApiDataTypes {
    AllCurrencies(Vec<NormalizedCurrency>),
    AllInstruments(Vec<NormalizedInstrument>)
}

impl NormalizedRestApiDataTypes {
    pub fn take_currencies(self) -> Option<Vec<NormalizedCurrency>> {
        match self {
            NormalizedRestApiDataTypes::AllCurrencies(vals) => Some(vals),
            _ => None
        }
    }

    pub fn take_instruments(self) -> Option<Vec<NormalizedInstrument>> {
        match self {
            NormalizedRestApiDataTypes::AllInstruments(vals) => Some(vals),
            _ => None
        }
    }
}
