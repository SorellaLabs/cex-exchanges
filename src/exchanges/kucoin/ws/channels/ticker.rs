use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::warn;

use crate::{kucoin::KucoinTradingPair, normalized::types::NormalizedQuote, CexExchange};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct KucoinTicker {
    #[serde(rename = "type")]
    pub kind:    String,
    pub topic:   KucoinTradingPair,
    pub subject: String,
    pub data:    KucoinTickerInner
}

impl KucoinTicker {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:   CexExchange::Kucoin,
            pair:       self.topic.normalize(),
            time:       DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64),
            ask_amount: self.data.best_ask_size,
            ask_price:  self.data.best_ask_price,
            bid_amount: self.data.best_bid_size,
            bid_price:  self.data.best_bid_price,
            quote_id:   Some(self.data.sequence.to_string())
        }
    }
}

impl PartialEq<NormalizedQuote> for KucoinTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Kucoin
            && other.pair == self.topic.normalize()
            && other.time == DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64)
            && other.ask_amount == self.data.best_ask_size
            && other.ask_price == self.data.best_ask_price
            && other.bid_amount == self.data.best_bid_size
            && other.bid_price == self.data.best_bid_price
            && other.quote_id == Some(self.data.sequence.to_string());

        if !equals {
            warn!(target: "cex-exchanges::kucoin", "kucoin ticker: {:?}", self);
            warn!(target: "cex-exchanges::kucoin", "normalized quote: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct KucoinTickerInner {
    #[serde_as(as = "DisplayFromStr")]
    pub sequence:       u64,
    #[serde_as(as = "DisplayFromStr")]
    pub price:          f64,
    #[serde_as(as = "DisplayFromStr")]
    pub size:           f64,
    #[serde(rename = "bestAsk")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_price: f64,
    #[serde(rename = "bestAskSize")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_size:  f64,
    #[serde(rename = "bestBid")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_price: f64,
    #[serde(rename = "bestBidSize")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_size:  f64,
    #[serde(rename = "time")]
    pub timestamp:      u64
}
