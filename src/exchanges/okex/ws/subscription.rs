use std::collections::HashSet;

use serde::Serialize;

use super::channels::OkexWsChannel;
use crate::{okex::OkexTradingPair, traits::SpecificWsSubscription};

#[derive(Debug, Clone, Serialize)]
pub struct OkexSubscription {
    op:   String,
    args: Vec<OkexSubscriptionInner>
}

impl Default for OkexSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl OkexSubscription {
    pub(crate) fn needs_business_ws(&self) -> bool {
        self.args.iter().any(|arg| arg.channel == "trades-all")
    }

    pub fn new_single_channel(channel: OkexWsChannel) -> Self {
        OkexSubscription { op: "subscribe".to_string(), args: channel.into() }
    }

    pub fn new() -> Self {
        OkexSubscription { op: "subscribe".to_string(), args: Vec::new() }
    }
}

impl SpecificWsSubscription for OkexSubscription {
    type TradingPair = OkexTradingPair;
    type WsChannel = OkexWsChannel;

    fn add_channel(&mut self, channel: Self::WsChannel) {
        let new: Vec<_> = channel.into();
        self.args.extend(new);
    }

    fn remove_pair(&mut self, pair: &Self::TradingPair) -> bool {
        self.args.retain(|p| &p.trading_pair != pair);

        self.args.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
struct OkexSubscriptionInner {
    channel:      String,
    #[serde(rename = "instId")]
    trading_pair: OkexTradingPair
}

impl From<OkexWsChannel> for Vec<OkexSubscriptionInner> {
    fn from(val: OkexWsChannel) -> Self {
        let name = val.to_string();

        let all_pairs: Vec<_> = match val {
            OkexWsChannel::TradesAll(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            OkexWsChannel::BookTicker(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        all_pairs
            .into_iter()
            .map(|p| OkexSubscriptionInner { channel: name.clone(), trading_pair: p })
            .collect()
    }
}
