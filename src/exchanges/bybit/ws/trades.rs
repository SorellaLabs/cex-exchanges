use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{bybit::pairs::BybitTradingPair, normalized::types::NormalizedTrade},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTrade {
    pub topic:             String,
    #[serde(rename = "type")]
    pub kind:              String,
    #[serde(rename = "ts")]
    pub request_timestamp: u64,
    pub data:              BybitTradeInner
}

impl BybitTrade {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Bybit,
            pair:     self.data.pair.normalize(),
            time:     DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64),
            side:     self.data.side.to_lowercase(),
            price:    self.data.price,
            amount:   self.data.amount,
            trade_id: Some(self.data.trade_id.to_string())
        }
    }
}

impl PartialEq<NormalizedTrade> for BybitTrade {
    fn eq(&self, other: &NormalizedTrade) -> bool {
        let equals = other.exchange == CexExchange::Bybit
            && other.pair == self.data.pair.normalize()
            && other.time == DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64)
            && other.side == self.data.side.to_lowercase()
            && other.price == self.data.price
            && other.amount == self.data.amount
            && other.trade_id.as_ref().unwrap() == &self.data.trade_id.to_string();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTradeInner {
    #[serde(rename = "T")]
    pub timestamp: u64,
    #[serde(rename = "s")]
    pub pair: BybitTradingPair,
    #[serde(rename = "S")]
    pub side: String,
    #[serde(rename = "v")]
    #[serde_as(as = "DisplayFromStr")]
    pub amount: f64,
    #[serde(rename = "p")]
    #[serde_as(as = "DisplayFromStr")]
    pub price: f64,
    #[serde(rename = "L")]
    pub direction_of_price_change: String,
    #[serde(rename = "i")]
    pub trade_id: u64,
    #[serde(rename = "BT")]
    pub is_block_trade_order: bool
}
