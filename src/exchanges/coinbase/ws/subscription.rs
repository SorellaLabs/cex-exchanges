use std::collections::HashSet;

use serde::Serialize;

use super::channels::CoinbaseWsChannel;
use crate::{coinbase::CoinbaseTradingPair, traits::SpecificWsSubscription};

#[derive(Debug, Clone, Serialize)]
pub struct CoinbaseSubscription {
    #[serde(rename = "type")]
    sub_name: String,
    channels: Vec<CoinbaseSubscriptionInner>
}

impl Default for CoinbaseSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinbaseSubscription {
    pub fn new() -> Self {
        CoinbaseSubscription { sub_name: "subscribe".to_string(), channels: Vec::new() }
    }

    pub fn new_single_channel(channel: CoinbaseWsChannel) -> Self {
        CoinbaseSubscription { sub_name: "subscribe".to_string(), channels: vec![channel.into()] }
    }
}

impl SpecificWsSubscription for CoinbaseSubscription {
    type TradingPair = CoinbaseTradingPair;
    type WsChannel = CoinbaseWsChannel;

    fn add_channel(&mut self, channel: Self::WsChannel) {
        self.channels.push(channel.into());
    }

    fn remove_pair(&mut self, pair: &Self::TradingPair) -> bool {
        self.channels.iter_mut().for_each(|sub| {
            sub.remove_pair(pair);
        });
        self.channels.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
struct CoinbaseSubscriptionInner {
    name:        String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    product_ids: Vec<CoinbaseTradingPair>
}

impl CoinbaseSubscriptionInner {
    fn remove_pair(&mut self, pair: &CoinbaseTradingPair) -> bool {
        let pre = self.product_ids.len();
        self.product_ids.retain(|p| p != pair);

        self.product_ids.len() < pre
    }
}

impl From<CoinbaseWsChannel> for CoinbaseSubscriptionInner {
    fn from(value: CoinbaseWsChannel) -> Self {
        let name = value.to_string();
        match value {
            CoinbaseWsChannel::Status => CoinbaseSubscriptionInner { name, product_ids: Vec::new() },
            CoinbaseWsChannel::Matches(pairs) => CoinbaseSubscriptionInner {
                name,
                product_ids: pairs
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect()
            },
            CoinbaseWsChannel::Ticker(pairs) => CoinbaseSubscriptionInner {
                name,
                product_ids: pairs
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect()
            }
        }
    }
}
