use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{coinbase::pairs::CoinbaseTradingPair, normalized::types::NormalizedQuote},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseTicker {
    pub sequence:      Option<u64>,
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
    pub side:          Option<String>,
    #[serde(default = "Utc::now")]
    pub time:          DateTime<Utc>,
    pub trade_id:      Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last_size:     Option<f64>
}

impl CoinbaseTicker {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:   CexExchange::Coinbase,
            pair:       self.product_id.normalize(),
            time:       self.time,
            ask_amount: self.best_ask_size,
            ask_price:  self.best_ask,
            bid_amount: self.best_bid_size,
            bid_price:  self.best_bid,
            quote_id:   self.trade_id.map(|t| t.to_string())
        }
    }
}

impl PartialEq<NormalizedQuote> for CoinbaseTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Coinbase
            && other.pair == self.product_id.normalize()
            && other.time == self.time
            && other.ask_amount == self.best_ask_size
            && other.ask_price == self.best_ask
            && other.bid_amount == self.best_bid_size
            && other.bid_price == self.best_bid
            && other.quote_id == self.trade_id.map(|t| t.to_string());

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
