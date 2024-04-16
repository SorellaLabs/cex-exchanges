use serde::{Deserialize, Serialize};

use crate::exchanges::normalized::CexExchange;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTradingPair {
    pub exchange:  CexExchange,
    pub base:      String,
    pub quote:     String,
    pub delimeter: Option<char>
}

impl NormalizedTradingPair {
    pub fn new(exchange: CexExchange, base: &str, quote: &str, delimeter: Option<char>) -> Self {
        Self { base: base.to_uppercase().to_string(), quote: quote.to_uppercase().to_string(), exchange, delimeter }
    }
}
