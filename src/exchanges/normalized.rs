use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedExchange {
    pub exchange: CexExchange,
    pub url:      String
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
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
