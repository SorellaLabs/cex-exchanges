use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::warn;

use crate::{
    bybit::BybitTradingPair,
    normalized::types::{NormalizedQuote, TimeOrUpdateId},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitOrderbook {
    pub topic:             String,
    #[serde(rename = "type")]
    pub kind:              String,
    #[serde(rename = "ts")]
    pub request_timestamp: u64,
    #[serde(rename = "cts")]
    pub timestamp:         u64,
    pub data:              BybitOrderbookInner
}

impl BybitOrderbook {
    pub fn normalize(self) -> Option<NormalizedQuote> {
        let bid = self.data.best_bid.first();
        let ask = self.data.best_ask.first();
        if let (Some(b), Some(a)) = (bid.as_ref(), ask.as_ref()) {
            Some(NormalizedQuote {
                exchange:           CexExchange::Bybit,
                pair:               self.data.symbol.normalize(),
                ask_amount:         a.amount,
                ask_price:          a.price,
                bid_amount:         b.amount,
                bid_price:          b.price,
                orderbook_ids_time: TimeOrUpdateId::new()
                    .with_time(DateTime::<Utc>::from_timestamp_millis(self.timestamp as i64).unwrap())
                    .with_first_update_id(self.data.update_id)
            })
        } else {
            None
        }
    }
}

impl PartialEq<Vec<NormalizedQuote>> for BybitOrderbook {
    fn eq(&self, other: &Vec<NormalizedQuote>) -> bool {
        if other.is_empty() {
            return true
        }
        let bid = self.data.best_bid.first();
        let ask = self.data.best_ask.first();

        let equals = other.iter().any(|other_data| {
            let data_bids = if let (Some(b), Some(a)) = (bid.as_ref(), ask.as_ref()) {
                other_data.ask_amount == a.amount
                    && other_data.ask_price == a.price
                    && other_data.bid_amount == b.amount
                    && other_data.bid_price == b.price
            } else {
                true
            };
            other_data.exchange == CexExchange::Bybit
                && other_data.pair == self.data.symbol.normalize()
                && other_data.orderbook_ids_time
                    == TimeOrUpdateId::new()
                        .with_time(DateTime::<Utc>::from_timestamp_millis(self.timestamp as i64).unwrap())
                        .with_first_update_id(self.data.update_id)
                && data_bids
        });

        if !equals {
            warn!(target: "cex-exchanges::bybit", "bybit orderbook: {:?}", self);
            warn!(target: "cex-exchanges::bybit", "normalized quote: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitOrderbookInner {
    #[serde(rename = "s")]
    pub symbol:         BybitTradingPair,
    #[serde(rename = "b")]
    pub best_bid:       Vec<BybitBidAsk>,
    #[serde(rename = "a")]
    pub best_ask:       Vec<BybitBidAsk>,
    #[serde(rename = "u")]
    pub update_id:      u64,
    #[serde(rename = "seq")]
    pub cross_sequence: u64
}

#[derive(Debug, Serialize, Clone, PartialEq, PartialOrd)]
pub struct BybitBidAsk {
    pub price:  f64,
    pub amount: f64
}

impl<'de> Deserialize<'de> for BybitBidAsk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let mut vals = Vec::<String>::deserialize(deserializer)?.into_iter();
        let price = vals
            .next()
            .ok_or(eyre::ErrReport::msg("no value in bid/ask vec".to_string()))
            .map_err(serde::de::Error::custom)?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let amount = vals
            .next()
            .ok_or(eyre::ErrReport::msg("no value in bid/ask vec".to_string()))
            .map_err(serde::de::Error::custom)?
            .parse()
            .map_err(serde::de::Error::custom)?;

        Ok(Self { price, amount })
    }
}
