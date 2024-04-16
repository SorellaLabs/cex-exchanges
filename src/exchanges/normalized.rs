use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct NormalizedExchange {
    pub exchange: CexExchange,
    pub url:      String
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CexExchange {
    Coinbase
}

impl Display for CexExchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CexExchange::Coinbase => write!(f, "coinbase")
        }
    }
}
