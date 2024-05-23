use std::collections::HashSet;

use serde::Serialize;

use super::channels::{BinanceWsChannel, BinanceWsChannelKind};
use crate::binance::BinanceTradingPair;

#[derive(Debug, Clone, Serialize)]
pub struct BinanceSubscription {
    method: String,
    params: Vec<BinanceSubscriptionInner>,
    id:     u64
}

impl BinanceSubscription {
    pub fn new() -> Self {
        BinanceSubscription { method: "SUBSCRIBE".to_string(), params: Vec::new(), id: 1 }
    }

    pub fn add_channel(&mut self, channel: BinanceWsChannel) {
        let new: Vec<BinanceSubscriptionInner> = channel.into();
        self.params.extend(new);
    }

    pub fn remove_pair(&mut self, pair: &BinanceTradingPair) -> bool {
        self.params.retain(|p| &p.trading_pair != pair);

        self.params.is_empty()
    }
}

impl Default for BinanceSubscription {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct BinanceSubscriptionInner {
    channel:      BinanceWsChannelKind,
    trading_pair: BinanceTradingPair
}

impl Serialize for BinanceSubscriptionInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        format!("{}@{}", self.trading_pair.0.to_lowercase(), self.channel).serialize(serializer)
    }
}

impl From<BinanceWsChannel> for Vec<BinanceSubscriptionInner> {
    fn from(val: BinanceWsChannel) -> Self {
        let channel = (&val).into();

        let all_pairs: Vec<_> = match val {
            BinanceWsChannel::Trade(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            BinanceWsChannel::BookTicker(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        all_pairs
            .into_iter()
            .map(|p| BinanceSubscriptionInner { channel, trading_pair: p })
            .collect()
    }
}
