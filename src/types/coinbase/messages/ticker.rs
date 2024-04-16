use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::normalized::CexExchange,
    types::{coinbase::pairs::CoinbaseTradingPair, normalized::quotes::NormalizedQuote}
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseTickerMessage {
    pub sequence:      u64,
    pub product_id:    CoinbaseTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    pub price:         f64,
    #[serde_as(as = "DisplayFromStr")]
    pub open_24h:      f64,
    #[serde_as(as = "DisplayFromStr")]
    pub low_24h:       f64,
    #[serde_as(as = "DisplayFromStr")]
    pub high_24h:      f64,
    #[serde_as(as = "DisplayFromStr")]
    pub volume_30d:    f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid:      f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_size: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask:      f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_size: f64,
    pub side:          String,
    pub time:          DateTime<Utc>,
    pub trade_id:      u64,
    #[serde_as(as = "DisplayFromStr")]
    pub last_size:     f64
}

impl CoinbaseTickerMessage {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:   CexExchange::Coinbase,
            pair:       self.product_id.normalize(),
            time:       self.time,
            ask_amount: self.best_ask_size,
            ask_price:  self.best_ask,
            bid_amount: self.best_bid_size,
            bid_price:  self.best_bid,
            quote_id:   Some(self.trade_id.to_string())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseTickerMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();

        normalized.exchange == CexExchange::Coinbase
            && normalized.pair == self.product_id.normalize()
            && normalized.time == self.time
            && normalized.ask_amount == self.best_ask_size
            && normalized.ask_price == self.best_ask
            && normalized.bid_amount == self.best_bid_size
            && normalized.bid_price == self.best_bid
            && normalized.quote_id.unwrap() == self.trade_id.to_string()
    }
}
