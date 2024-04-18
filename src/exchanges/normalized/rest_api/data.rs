use serde::Serialize;

use crate::normalized::types::{NormalizedCurrency, NormalizedInstrument};

#[derive(Debug, Clone, Serialize)]
pub enum NormalizedRestApiDataTypes {
    AllCurrencies(Vec<NormalizedCurrency>),
    AllInstruments(Vec<NormalizedInstrument>)
}
