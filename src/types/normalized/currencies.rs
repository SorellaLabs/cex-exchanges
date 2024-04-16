use serde::{Deserialize, Serialize};

use crate::{exchanges::CexExchange, types::blockchain::Blockchain};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedCurrency {
    pub exchange:     CexExchange,
    pub symbol:       String,
    pub name:         String,
    pub display_name: Option<String>,
    pub status:       String,
    /// (blockchain, Option<address>)
    pub blockchains:  Vec<(Blockchain, Option<String>)>
}
