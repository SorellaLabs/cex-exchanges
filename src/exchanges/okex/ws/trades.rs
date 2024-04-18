use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{normalized::types::NormalizedTrade, okex::pairs::OkexTradingPair},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OkexTradesAllMessage {
    #[serde(rename = "instId")]
    pub pair:       OkexTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "px")]
    pub price:      f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "sz")]
    pub quantity:   f64,
    #[serde(rename = "tradeId")]
    pub trade_id:   String,
    pub side:       String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "ts")]
    pub trade_time: u64
}

impl OkexTradesAllMessage {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Okex,
            pair:     self.pair.normalize(),
            time:     DateTime::from_timestamp_millis(self.trade_time as i64).unwrap(),
            side:     self.side.to_lowercase(),
            price:    self.price,
            amount:   self.quantity,
            trade_id: Some(self.trade_id.to_string())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for OkexTradesAllMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();

        normalized.exchange == CexExchange::Okex
            && normalized.pair == self.pair.normalize()
            && normalized.time == DateTime::from_timestamp_millis(self.trade_time as i64).unwrap()
            && normalized.side == self.side.to_lowercase()
            && normalized.price == self.price
            && normalized.amount == self.quantity
            && normalized.trade_id.unwrap() == self.trade_id.to_string()
    }
}
