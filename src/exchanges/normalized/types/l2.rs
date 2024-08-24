use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{pairs::NormalizedTradingPair, NormalizedQuote};
use crate::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct NormalizedL2 {
    pub exchange:  CexExchange,
    pub pair:      NormalizedTradingPair,
    pub time:      DateTime<Utc>,
    pub bids:      Vec<BidAsk>,
    pub asks:      Vec<BidAsk>,
    pub update_id: Option<String>
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
                exchange:   self.exchange,
                pair:       self.pair.clone(),
                time:       self.time,
                ask_amount: ask.amount,
                ask_price:  ask.price,
                bid_amount: bid.amount,
                bid_price:  bid.price,
                quote_id:   self.update_id.clone()
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
