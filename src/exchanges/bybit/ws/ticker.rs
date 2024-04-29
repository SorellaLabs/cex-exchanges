use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{bybit::BybitTradingPair, normalized::types::NormalizedQuote, CexExchange};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTicker {
    pub topic:             String,
    #[serde(rename = "type")]
    pub kind:              String,
    #[serde(rename = "ts")]
    pub request_timestamp: u64,
    #[serde(rename = "cts")]
    pub timestamp:         u64,
    pub data:              BybitTickerInner
}

impl BybitTicker {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:   CexExchange::Bybit,
            pair:       self.data.symbol.normalize(),
            time:       DateTime::<Utc>::from_timestamp_nanos(self.timestamp as i64),
            ask_amount: self.data.best_ask.amount,
            ask_price:  self.data.best_ask.price,
            bid_amount: self.data.best_bid.amount,
            bid_price:  self.data.best_bid.price,
            quote_id:   Some(self.data.update_id.to_string())
        }
    }
}

impl PartialEq<NormalizedQuote> for BybitTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Bybit
            && other.pair == self.data.symbol.normalize()
            && other.time == DateTime::<Utc>::from_timestamp_nanos(self.timestamp as i64)
            && other.ask_amount == self.data.best_ask.amount
            && other.ask_price == self.data.best_ask.price
            && other.bid_amount == self.data.best_bid.amount
            && other.bid_price == self.data.best_bid.price
            && other.quote_id == Some(self.data.update_id.to_string());

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTickerInner {
    #[serde(rename = "s")]
    pub symbol:         BybitTradingPair,
    #[serde(rename = "b")]
    pub best_bid:       BybitBidAsk,
    #[serde(rename = "a")]
    pub best_ask:       BybitBidAsk,
    #[serde(rename = "u")]
    pub update_id:      u64,
    #[serde(rename = "seq")]
    pub cross_sequence: u64
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitBidAsk {
    #[serde_as(as = "DisplayFromStr")]
    pub price:  f64,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: f64
}
