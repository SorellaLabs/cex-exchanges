use std::collections::HashMap;

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::channels::{KucoinWsChannel, KucoinWsChannelKind};
use crate::kucoin::KucoinTradingPair;

#[derive(Debug, Default, Clone)]
pub struct KucoinMultiSubscription {
    subscriptions: HashMap<KucoinWsChannelKind, KucoinSubscription>
}

impl KucoinMultiSubscription {
    pub fn add_channel(&mut self, channel: KucoinWsChannel) {
        match channel {
            KucoinWsChannel::Match(pairs) => self
                .subscriptions
                .entry(KucoinWsChannelKind::Match)
                .or_insert(KucoinSubscription::new(KucoinWsChannelKind::Match))
                .add_pairs(pairs),
            KucoinWsChannel::Ticker(pairs) => self
                .subscriptions
                .entry(KucoinWsChannelKind::Ticker)
                .or_insert(KucoinSubscription::new(KucoinWsChannelKind::Ticker))
                .add_pairs(pairs)
        }
    }

    pub fn all_subscriptions(self) -> Vec<KucoinSubscription> {
        self.subscriptions.into_values().collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KucoinSubscription {
    /// some random number?
    id:              u64,
    #[serde(rename = "type")]
    method:          String,
    topic:           KucoinSubscriptionInner,
    #[serde(rename = "privateChannel")]
    private_channel: bool,
    response:        bool
}

impl KucoinSubscription {
    pub fn new(channel: KucoinWsChannelKind) -> Self {
        let mut rng = rand::thread_rng();

        KucoinSubscription {
            method:          "subscribe".to_string(),
            id:              rng.gen(),
            topic:           KucoinSubscriptionInner::new(channel),
            private_channel: false,
            response:        false
        }
    }

    pub fn add_pairs(&mut self, pairs: Vec<KucoinTradingPair>) {
        self.topic.trading_pairs.extend(pairs)
    }

    pub fn remove_pair(&mut self, pair: &KucoinTradingPair) -> bool {
        self.topic.trading_pairs.retain(|p| p != pair);

        self.topic.trading_pairs.is_empty()
    }
}

#[derive(Debug, Clone)]
struct KucoinSubscriptionInner {
    channel:       KucoinWsChannelKind,
    trading_pairs: Vec<KucoinTradingPair>
}

impl KucoinSubscriptionInner {
    fn new(channel: KucoinWsChannelKind) -> Self {
        Self { channel, trading_pairs: Vec::new() }
    }
}

impl Serialize for KucoinSubscriptionInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let pairs = self
            .trading_pairs
            .iter()
            .map(|pair| pair.0.to_uppercase())
            .collect::<Vec<_>>()
            .join(",");
        format!("/market/{}:{}", self.channel, pairs).serialize(serializer)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KucoinWsEndpointResponse {
    #[allow(unused)]
    code: String,
    data: KucoinWsEndpointDataResponse
}

impl KucoinWsEndpointResponse {
    pub fn get_ws_endpoint(&self) -> Option<String> {
        self.data
            .instance_servers
            .iter()
            .find(|server| &server.protocol == "websocket")
            .map(|server| &server.endpoint)
            .cloned()
    }

    pub fn get_token(&self) -> String {
        self.data.token.clone()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct KucoinWsEndpointDataResponse {
    token:            String,
    #[serde(rename = "instanceServers")]
    instance_servers: Vec<KucoinWsEndpointInstanceServersResponse>
}

#[derive(Debug, Clone, Deserialize)]
struct KucoinWsEndpointInstanceServersResponse {
    endpoint:      String,
    #[allow(unused)]
    encrypt:       bool,
    protocol:      String,
    #[allow(unused)]
    #[serde(rename = "pingInterval")]
    ping_interval: u64,
    #[allow(unused)]
    #[serde(rename = "pingTimeout")]
    ping_timeout:  u64
}
