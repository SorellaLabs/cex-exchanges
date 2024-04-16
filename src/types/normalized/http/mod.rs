use super::currencies::NormalizedCurrency;

pub mod combined;

#[derive(Debug, Clone)]
pub enum NormalizedHttpDataTypes {
    AllCurrencies(Vec<NormalizedCurrency>)
}
