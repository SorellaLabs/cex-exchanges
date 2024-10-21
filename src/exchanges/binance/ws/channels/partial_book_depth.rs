use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::warn;

use crate::{
    binance::BinanceTradingPair,
    normalized::types::{BidAsk, NormalizedL2, TimeOrUpdateId},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BinancePartialBookDepth {
    pub pair:                BinanceTradingPair,
    pub bids:                Vec<Vec<f64>>,
    pub asks:                Vec<Vec<f64>>,
    pub orderbook_update_id: u64
}

impl BinancePartialBookDepth {
    pub fn normalize(self) -> NormalizedL2 {
        NormalizedL2 {
            exchange:           CexExchange::Binance,
            pair:               self.pair.normalize(),
            bids:               self
                .bids
                .into_iter()
                .map(|bid| BidAsk::new(bid[0], bid[1]))
                .collect(),
            asks:               self
                .asks
                .into_iter()
                .map(|ask| BidAsk::new(ask[0], ask[1]))
                .collect(),
            orderbook_ids_time: TimeOrUpdateId::new().with_first_update_id(self.orderbook_update_id)
        }
    }
}

impl PartialEq<NormalizedL2> for BinancePartialBookDepth {
    fn eq(&self, other: &NormalizedL2) -> bool {
        let our_bids = self
            .bids
            .iter()
            .map(|bid| BidAsk::new(bid[0], bid[1]))
            .collect::<Vec<_>>();

        let our_asks = self
            .asks
            .iter()
            .map(|ask| BidAsk::new(ask[0], ask[1]))
            .collect::<Vec<_>>();
        let equals = other.exchange == CexExchange::Binance
            && other.pair == self.pair.normalize()
            && other.bids.iter().all(|b| our_bids.contains(b))
            && other.bids.len() == our_bids.len()
            && other.asks.iter().all(|a| our_asks.contains(a))
            && other.asks.len() == our_asks.len()
            && other.orderbook_ids_time == TimeOrUpdateId::new().with_first_update_id(self.orderbook_update_id);

        if !equals {
            warn!(target: "cex-exchanges::binance", "binance diff depth: {:?}", self);
            warn!(target: "cex-exchanges::binance", "normalized l2: {:?}", other);
        }

        equals
    }
}
