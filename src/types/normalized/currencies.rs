use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::CexExchange, types::blockchain::Blockchain};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCurrency {
    pub exchange:      CexExchange,
    pub symbol:        String,
    pub name:          String,
    pub display_name:  Option<String>,
    pub min_size:      f64,
    pub max_precision: f64,
    pub status:        String,
    pub is_fiat:       bool,
    /// (blockchain, Option<address>)
    pub blockchains:   Vec<(Blockchain, Option<String>)>
}
