use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::pairs::NormalizedTradingPair;
use crate::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct NormalizedQuote {
    pub exchange:   CexExchange,
    pub pair:       NormalizedTradingPair,
    pub time:       DateTime<Utc>,
    pub ask_amount: f64,
    pub ask_price:  f64,
    pub bid_amount: f64,
    pub bid_price:  f64,
    pub quote_id:   Option<String>
}
