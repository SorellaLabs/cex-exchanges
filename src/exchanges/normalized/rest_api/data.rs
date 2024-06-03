use serde::Serialize;

use crate::{
    normalized::types::{NormalizedCurrency, NormalizedInstrument},
    ExchangeFilter
};

#[derive(Debug, Clone, Serialize)]
pub enum NormalizedRestApiDataTypes {
    AllCurrencies(Vec<NormalizedCurrency>),
    AllInstruments(Vec<NormalizedInstrument>)
}

impl NormalizedRestApiDataTypes {
    pub fn take_currencies<F>(self, filter: Option<F>) -> Option<Vec<NormalizedCurrency>>
    where
        F: ExchangeFilter<NormalizedCurrency>
    {
        match self {
            NormalizedRestApiDataTypes::AllCurrencies(mut vals) => {
                if let Some(f) = filter {
                    f.filter_matches(&mut vals);
                }
                Some(vals)
            }
            _ => None
        }
    }

    pub fn take_instruments<F>(self, filter: Option<F>) -> Option<Vec<NormalizedInstrument>>
    where
        F: ExchangeFilter<NormalizedInstrument>
    {
        match self {
            NormalizedRestApiDataTypes::AllInstruments(mut vals) => {
                if let Some(f) = filter {
                    f.filter_matches(&mut vals);
                }
                Some(vals)
            }
            _ => None
        }
    }
}
