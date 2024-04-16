use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::CexExchange,
    types::{coinbase::pairs::CoinbaseTradingPair, normalized::trades::NormalizedTrade}
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseMatchesMessage {
    pub trade_id:       u64,
    pub sequence:       u64,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub time:           DateTime<Utc>,
    pub product_id:     CoinbaseTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    pub size:           f64,
    #[serde_as(as = "DisplayFromStr")]
    pub price:          f64,
    pub side:           String
}

impl CoinbaseMatchesMessage {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Coinbase,
            pair:     self.product_id.normalize(),
            time:     self.time,
            side:     self.side.to_lowercase(),
            price:    self.price,
            amount:   self.size,
            trade_id: Some(self.trade_id.to_string())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseMatchesMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();

        normalized.exchange == CexExchange::Coinbase
            && normalized.pair == self.product_id.normalize()
            && normalized.time == self.time
            && normalized.side == self.side.to_lowercase()
            && normalized.price == self.price
            && normalized.amount == self.size
            && normalized.trade_id.unwrap() == self.trade_id.to_string()
    }
}
