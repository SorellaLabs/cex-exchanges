use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::warn;

use crate::{
    binance::BinanceTradingPair,
    normalized::types::{NormalizedQuote, TimeOrUpdateId},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BinanceBookTicker {
    #[serde(rename = "s")]
    pub pair:                BinanceTradingPair,
    #[serde(rename = "A")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_amt:        f64,
    #[serde(rename = "a")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_price:      f64,
    #[serde(rename = "B")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_amt:        f64,
    #[serde(rename = "b")]
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_price:      f64,
    #[serde(rename = "u")]
    pub orderbook_update_id: u64,
    #[serde(default = "Utc::now")]
    pub local_update_time:   DateTime<Utc>
}

impl BinanceBookTicker {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:           CexExchange::Binance,
            pair:               self.pair.normalize(),
            ask_amount:         self.best_ask_amt,
            ask_price:          self.best_ask_price,
            bid_amount:         self.best_bid_amt,
            bid_price:          self.best_bid_price,
            orderbook_ids_time: TimeOrUpdateId::new()
                .with_time(self.local_update_time)
                .with_first_update_id(self.orderbook_update_id)
        }
    }
}

impl PartialEq<NormalizedQuote> for BinanceBookTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Binance
            && other.pair == self.pair.normalize()
            && other.ask_amount == self.best_ask_amt
            && other.ask_price == self.best_ask_price
            && other.bid_amount == self.best_bid_amt
            && other.bid_price == self.best_bid_price
            && other.orderbook_ids_time
                == TimeOrUpdateId::new()
                    .with_time(self.local_update_time)
                    .with_first_update_id(self.orderbook_update_id);

        if !equals {
            warn!(target: "cex-exchanges::binance", "binance book ticker: {:?}", self);
            warn!(target: "cex-exchanges::binance", "normalized quote: {:?}", other);
        }

        equals
    }
}
