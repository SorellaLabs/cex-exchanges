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
    /// (blockchain, Option<address>)
    pub blockchains:  Vec<(Blockchain, Option<String>)>
}
