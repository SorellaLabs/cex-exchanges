use crate::normalized::types::{NormalizedCurrency, NormalizedInstrument};

#[derive(Debug, Clone)]
pub enum NormalizedRestApiDataTypes {
    AllCurrencies(Vec<NormalizedCurrency>),
    AllInstruments(Vec<NormalizedInstrument>)
}
