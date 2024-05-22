use serde::{Deserialize, Serialize};

use super::Blockchain;
use crate::exchanges::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct NormalizedCurrency {
    pub exchange:     CexExchange,
    pub symbol:       String,
    pub name:         String,
    pub display_name: Option<String>,
    pub status:       String,
    pub blockchains:  Vec<BlockchainCurrency>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BlockchainCurrency {
    pub blockchain:       Blockchain,
    pub address:          Option<String>,
    /// (true & wrapped_currency) == None -> tbd
    pub is_wrapped:       bool,
    pub wrapped_currency: Option<WrappedCurrency>
}

impl BlockchainCurrency {
    pub fn wrapped(&mut self, is_wrapped: bool) {
        self.is_wrapped = is_wrapped;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct WrappedCurrency {
    pub symbol: String,
    pub name:   String
}
