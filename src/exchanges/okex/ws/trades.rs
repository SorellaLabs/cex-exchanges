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

impl PartialEq<NormalizedTrade> for OkexTradesAllMessage {
    fn eq(&self, other: &NormalizedTrade) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.pair == self.pair.normalize()
            && other.time == DateTime::from_timestamp_millis(self.trade_time as i64).unwrap()
            && other.side == self.side.to_lowercase()
            && other.price == self.price
            && other.amount == self.quantity
            && other.trade_id.as_ref().unwrap() == &self.trade_id.to_string();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
