use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::pairs::NormalizedTradingPair;
use crate::exchanges::normalized::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTrade {
    pub exchange: CexExchange,
    pub pair:     NormalizedTradingPair,
    pub time:     DateTime<Utc>,
    pub side:     String,
    pub price:    f64,
    pub amount:   f64,
    pub trade_id: Option<String>
}
