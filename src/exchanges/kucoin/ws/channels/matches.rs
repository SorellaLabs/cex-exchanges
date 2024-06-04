use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::warn;

use crate::{kucoin::KucoinTradingPair, normalized::types::NormalizedTrade, CexExchange};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct KucoinMatch {
    #[serde(rename = "type")]
    pub kind:    String,
    pub topic:   String,
    pub subject: String,
    pub data:    KucoinMatchInner
}

impl KucoinMatch {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Kucoin,
            pair:     self.data.symbol.normalize(),
            time:     DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64),
            side:     self.data.side.to_lowercase(),
            price:    self.data.price,
            amount:   self.data.size,
            trade_id: Some(self.data.trade_id)
        }
    }
}

impl PartialEq<NormalizedTrade> for KucoinMatch {
    fn eq(&self, other: &NormalizedTrade) -> bool {
        let equals = other.exchange == CexExchange::Kucoin
            && other.pair == self.data.symbol.normalize()
            && other.time == DateTime::<Utc>::from_timestamp_nanos(self.data.timestamp as i64)
            && other.side == self.data.side.to_lowercase()
            && other.price == self.data.price
            && other.amount == self.data.size
            && other.trade_id.as_ref().unwrap() == &self.data.trade_id.to_string();

        if !equals {
            warn!(target: "cex-exchanges::kucoin", "kucoin match: {:?}", self);
            warn!(target: "cex-exchanges::kucoin", "normalized trade: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct KucoinMatchInner {
    #[serde_as(as = "DisplayFromStr")]
    pub sequence:       u64,
    #[serde(rename = "type")]
    pub kind:           String,
    pub symbol:         KucoinTradingPair,
    pub side:           String,
    #[serde_as(as = "DisplayFromStr")]
    pub price:          f64,
    #[serde_as(as = "DisplayFromStr")]
    pub size:           f64,
    #[serde(rename = "tradeId")]
    pub trade_id:       String,
    #[serde(rename = "takerOrderId")]
    pub taker_order_id: String,
    #[serde(rename = "makerOrderId")]
    pub maker_order_id: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "time")]
    pub timestamp:      u64
}
