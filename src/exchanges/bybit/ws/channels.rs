use std::{collections::HashSet, fmt::Display};

use serde::Serialize;

use crate::{
    exchanges::{
        kucoin::pairs::BybitTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BybitWsChannel {
    Trade(Vec<BybitTradingPair>),
    BookTicker(Vec<BybitTradingPair>)
}

impl BybitWsChannel {
    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Bybit))
            .collect();

        Self::new_from_normalized(normalized, BybitWsChannel::Trade(Vec::new()))
    }

    /// builds the book ticker channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    pub fn new_book_ticker(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Bybit))
            .collect();

        Self::new_from_normalized(normalized, BybitWsChannel::BookTicker(Vec::new()))
    }

    pub(crate) fn new_from_normalized(pairs: Vec<NormalizedTradingPair>, kind: BybitWsChannel) -> eyre::Result<Self> {
        match kind {
            BybitWsChannel::Trade(_) => Ok(BybitWsChannel::Trade(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            BybitWsChannel::BookTicker(_) => Ok(BybitWsChannel::BookTicker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }

    pub fn count_entries(&self) -> usize {
        match self {
            BybitWsChannel::Trade(vals) => vals.len(),
            BybitWsChannel::BookTicker(vals) => vals.len()
        }
    }
}

impl Display for BybitWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BybitWsChannel::Trade(_) => write!(f, "trade"),
            BybitWsChannel::BookTicker(_) => write!(f, "bookTicker")
        }
    }
}

impl TryFrom<String> for BybitWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" => Ok(Self::Trade(Vec::new())),
            "bookTicker" => Ok(Self::BookTicker(Vec::new())),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl TryFrom<NormalizedWsChannels> for BybitWsChannel {
    type Error = eyre::ErrReport;

    fn try_from(value: NormalizedWsChannels) -> Result<Self, Self::Error> {
        match value {
            NormalizedWsChannels::Trades(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(BybitWsChannel::Trade(norm_pairs))
            }

            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BybitWsChannelKind {
    Trade,
    BookTicker
}

impl Display for BybitWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BybitWsChannelKind::Trade => write!(f, "trade"),
            BybitWsChannelKind::BookTicker => write!(f, "bookTicker")
        }
    }
}

impl From<&BybitWsChannel> for BybitWsChannelKind {
    fn from(value: &BybitWsChannel) -> Self {
        match value {
            BybitWsChannel::Trade(_) => BybitWsChannelKind::Trade,
            BybitWsChannel::BookTicker(_) => BybitWsChannelKind::BookTicker
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BybitSubscription {
    method: String,
    params: Vec<BybitSubscriptionInner>,
    id:     u64
}

impl BybitSubscription {
    pub fn new() -> Self {
        BybitSubscription { method: "SUBSCRIBE".to_string(), params: Vec::new(), id: 1 }
    }

    pub fn add_channel(&mut self, channel: BybitWsChannel) {
        let new: Vec<BybitSubscriptionInner> = channel.into();
        self.params.extend(new);
    }
}

impl Default for BybitSubscription {
    fn default() -> Self {
        Self::new()
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
        format!("{}@{}", self.trading_pair.0.to_lowercase(), self.channel.to_string()).serialize(serializer)
    }
}

impl Into<Vec<BybitSubscriptionInner>> for BybitWsChannel {
    fn into(self) -> Vec<BybitSubscriptionInner> {
        let channel = (&self).into();

        let all_pairs: Vec<_> = match self {
            BybitWsChannel::Trade(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            BybitWsChannel::BookTicker(pairs) => pairs
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
