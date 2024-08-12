use std::collections::HashSet;

use serde::Serialize;

use super::channels::{BybitWsChannel, BybitWsChannelKind};
use crate::{bybit::BybitTradingPair, traits::SpecificWsSubscription};

#[derive(Debug, Default, Clone, Serialize)]
pub struct BybitSubscription {
    op:   String,
    args: Vec<BybitSubscriptionInner>
}

impl BybitSubscription {
    pub fn new() -> Self {
        BybitSubscription { op: "subscribe".to_string(), args: Vec::new() }
    }
}

impl SpecificWsSubscription for BybitSubscription {
    type TradingPair = BybitTradingPair;
    type WsChannel = BybitWsChannel;

    fn add_channel(&mut self, channel: Self::WsChannel) {
        let new: Vec<BybitSubscriptionInner> = channel.into();
        self.args.extend(new);
    }

    fn remove_pair(&mut self, pair: &Self::TradingPair) -> bool {
        self.args.retain(|p| &p.trading_pair != pair);

        self.args.is_empty()
    }
}

#[derive(Debug, Clone)]
struct BybitSubscriptionInner {
    channel:      BybitWsChannelKind,
    trading_pair: BybitTradingPair
}

impl Serialize for BybitSubscriptionInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        format!("{}.{}", self.channel, self.trading_pair.0.to_uppercase()).serialize(serializer)
    }
}

impl From<BybitWsChannel> for Vec<BybitSubscriptionInner> {
    fn from(val: BybitWsChannel) -> Self {
        let channel = (&val).into();

        let all_pairs: Vec<_> = match val {
            BybitWsChannel::Trade(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            BybitWsChannel::OrderbookL1(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        all_pairs
            .into_iter()
            .map(|p| BybitSubscriptionInner { channel, trading_pair: p })
            .collect()
    }
}
