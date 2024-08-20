use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::warn;

use crate::{
    binance::BinanceTradingPair,
    normalized::types::{BidAsk, NormalizedL2},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BinanceDiffDepth {
    #[serde(rename = "s")]
    pub pair: BinanceTradingPair,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "b")]
    #[serde_as(as = "Vec<Vec<DisplayFromStr>>")]
    pub bids: Vec<Vec<f64>>,
    #[serde(rename = "a")]
    #[serde_as(as = "Vec<Vec<DisplayFromStr>>")]
    pub asks: Vec<Vec<f64>>,
    #[serde(rename = "U")]
    pub first_orderbook_update_id: u64,
    #[serde(rename = "u")]
    pub last_orderbook_update_id: u64
}

impl BinanceDiffDepth {
    pub fn normalize(self) -> NormalizedL2 {
        NormalizedL2 {
            exchange:  CexExchange::Binance,
            pair:      self.pair.normalize(),
            time:      DateTime::from_timestamp_millis(self.event_time as i64).unwrap(),
            bids:      self
                .bids
                .into_iter()
                .map(|bid| BidAsk::new(bid[0], bid[1]))
                .collect(),
            asks:      self
                .asks
                .into_iter()
                .map(|ask| BidAsk::new(ask[0], ask[1]))
                .collect(),
            update_id: Some(format!("orderbook range: {} - {}", self.first_orderbook_update_id, self.last_orderbook_update_id))
        }
    }
}

impl PartialEq<NormalizedL2> for BinanceDiffDepth {
    fn eq(&self, other: &NormalizedL2) -> bool {
        let equals = other.exchange == CexExchange::Binance
            && other.pair == self.pair.normalize()
            && other.time == DateTime::from_timestamp_millis(self.event_time as i64).unwrap()
            && other.bids
                == self
                    .bids
                    .iter()
                    .map(|bid| BidAsk::new(bid[0], bid[1]))
                    .collect::<Vec<_>>()
            && other.asks
                == self
                    .asks
                    .iter()
                    .map(|ask| BidAsk::new(ask[0], ask[1]))
                    .collect::<Vec<_>>()
            && other.update_id == Some(format!("orderbook range: {} - {}", self.first_orderbook_update_id, self.last_orderbook_update_id));

        if !equals {
            warn!(target: "cex-exchanges::binance", "binance book ticker: {:?}", self);
            warn!(target: "cex-exchanges::binance", "normalized quote: {:?}", other);
        }

        equals
    }
}
