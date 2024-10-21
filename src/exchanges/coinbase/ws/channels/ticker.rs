use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::warn;

use crate::{
    exchanges::{coinbase::pairs::CoinbaseTradingPair, normalized::types::NormalizedQuote},
    normalized::types::TimeOrUpdateId,
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
    // #[serde(default = "Utc::now")]
    pub time:          DateTime<Utc>,
    pub trade_id:      Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last_size:     Option<f64>
}

impl CoinbaseTicker {
    pub fn normalize(self) -> NormalizedQuote {
        let mut orderbook_ids_time = TimeOrUpdateId::new().with_time(self.time);

        if let Some(id) = self.trade_id {
            orderbook_ids_time = orderbook_ids_time.with_first_update_id(id);
        }

        NormalizedQuote {
            exchange: CexExchange::Coinbase,
            pair: self.product_id.normalize(),
            ask_amount: self.best_ask_size,
            ask_price: self.best_ask,
            bid_amount: self.best_bid_size,
            bid_price: self.best_bid,
            orderbook_ids_time
        }
    }
}

impl PartialEq<NormalizedQuote> for CoinbaseTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let mut orderbook_ids_time = TimeOrUpdateId::new().with_time(self.time);

        if let Some(id) = self.trade_id {
            orderbook_ids_time = orderbook_ids_time.with_first_update_id(id);
        }

        let equals = other.exchange == CexExchange::Coinbase
            && other.pair == self.product_id.normalize()
            && other.ask_amount == self.best_ask_size
            && other.ask_price == self.best_ask
            && other.bid_amount == self.best_bid_size
            && other.bid_price == self.best_bid
            && other.orderbook_ids_time == orderbook_ids_time;

        if !equals {
            warn!(target: "cex-exchanges::coinbase", "coinbase ticker: {:?}", self);
            warn!(target: "cex-exchanges::coinbase", "normalized quote: {:?}", other);
        }

        equals
    }
}
