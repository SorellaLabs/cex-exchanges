use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{pairs::NormalizedTradingPair, NormalizedQuote};
use crate::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct NormalizedL2 {
    pub exchange:           CexExchange,
    pub pair:               NormalizedTradingPair,
    pub bids:               Vec<BidAsk>,
    pub asks:               Vec<BidAsk>,
    pub orderbook_ids_time: TimeOrUpdateId
}

impl NormalizedL2 {
    pub fn get_quote(&self) -> Option<NormalizedQuote> {
        if let (Some(bid), Some(ask)) = (
            self.bids
                .iter()
                .filter(|v| v.amount != 0.0)
                .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
            self.asks
                .iter()
                .filter(|v| v.amount != 0.0)
                .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
        ) {
            Some(NormalizedQuote {
                exchange:           self.exchange,
                pair:               self.pair.clone(),
                ask_amount:         ask.amount,
                ask_price:          ask.price,
                bid_amount:         bid.amount,
                bid_price:          bid.price,
                orderbook_ids_time: self.orderbook_ids_time.clone()
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BidAsk {
    pub price:  f64,
    pub amount: f64
}

impl BidAsk {
    pub fn new(price: f64, amount: f64) -> Self {
        Self { price, amount }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub struct TimeOrUpdateId {
    pub time:            Option<DateTime<Utc>>,
    pub first_update_id: Option<u64>,
    pub last_update_id:  Option<u64>
}

impl TimeOrUpdateId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_time(mut self, time: DateTime<Utc>) -> Self {
        self.time = Some(time);
        self
    }

    pub fn with_first_update_id(mut self, first_update_id: u64) -> Self {
        self.first_update_id = Some(first_update_id);
        self
    }

    pub fn with_last_update_id(mut self, last_update_id: u64) -> Self {
        self.last_update_id = Some(last_update_id);
        self
    }
}
