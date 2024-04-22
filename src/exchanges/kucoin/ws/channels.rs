use std::{collections::HashSet, fmt::Display};

use serde::Serialize;

use crate::{
    exchanges::{
        kucoin::pairs::KucoinTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KucoinWsChannel {
    Trade(Vec<KucoinTradingPair>),
    BookTicker(Vec<KucoinTradingPair>)
}

impl KucoinWsChannel {
    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Kucoin))
            .collect();

        Self::new_from_normalized(normalized, KucoinWsChannel::Trade(Vec::new()))
    }

    /// builds the book ticker channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    pub fn new_book_ticker(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Kucoin))
            .collect();

        Self::new_from_normalized(normalized, KucoinWsChannel::BookTicker(Vec::new()))
    }

    pub(crate) fn new_from_normalized(pairs: Vec<NormalizedTradingPair>, kind: KucoinWsChannel) -> eyre::Result<Self> {
        match kind {
            KucoinWsChannel::Trade(_) => Ok(KucoinWsChannel::Trade(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            KucoinWsChannel::BookTicker(_) => Ok(KucoinWsChannel::BookTicker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }

    pub fn count_entries(&self) -> usize {
        match self {
            KucoinWsChannel::Trade(vals) => vals.len(),
            KucoinWsChannel::BookTicker(vals) => vals.len()
        }
    }
}

impl Display for KucoinWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KucoinWsChannel::Trade(_) => write!(f, "trade"),
            KucoinWsChannel::BookTicker(_) => write!(f, "bookTicker")
        }
    }
}

impl TryFrom<String> for KucoinWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" => Ok(Self::Trade(Vec::new())),
            "bookTicker" => Ok(Self::BookTicker(Vec::new())),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl TryFrom<NormalizedWsChannels> for KucoinWsChannel {
    type Error = eyre::ErrReport;

    fn try_from(value: NormalizedWsChannels) -> Result<Self, Self::Error> {
        match value {
            NormalizedWsChannels::Trades(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(KucoinWsChannel::Trade(norm_pairs))
            }

            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KucoinWsChannelKind {
    Trade,
    BookTicker
}

impl Display for KucoinWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KucoinWsChannelKind::Trade => write!(f, "trade"),
            KucoinWsChannelKind::BookTicker => write!(f, "bookTicker")
        }
    }
}

impl From<&KucoinWsChannel> for KucoinWsChannelKind {
    fn from(value: &KucoinWsChannel) -> Self {
        match value {
            KucoinWsChannel::Trade(_) => KucoinWsChannelKind::Trade,
            KucoinWsChannel::BookTicker(_) => KucoinWsChannelKind::BookTicker
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KucoinSubscription {
    method: String,
    params: Vec<KucoinSubscriptionInner>,
    id:     u64
}

impl KucoinSubscription {
    pub fn new() -> Self {
        KucoinSubscription { method: "SUBSCRIBE".to_string(), params: Vec::new(), id: 1 }
    }

    pub fn add_channel(&mut self, channel: KucoinWsChannel) {
        let new: Vec<KucoinSubscriptionInner> = channel.into();
        self.params.extend(new);
    }
}

impl Default for KucoinSubscription {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct KucoinSubscriptionInner {
    channel:      KucoinWsChannelKind,
    trading_pair: KucoinTradingPair
}

impl Serialize for KucoinSubscriptionInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        format!("{}@{}", self.trading_pair.0.to_lowercase(), self.channel.to_string()).serialize(serializer)
    }
}

impl Into<Vec<KucoinSubscriptionInner>> for KucoinWsChannel {
    fn into(self) -> Vec<KucoinSubscriptionInner> {
        let channel = (&self).into();

        let all_pairs: Vec<_> = match self {
            KucoinWsChannel::Trade(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect(),
            KucoinWsChannel::BookTicker(pairs) => pairs
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        all_pairs
            .into_iter()
            .map(|p| KucoinSubscriptionInner { channel, trading_pair: p })
            .collect()
    }
}
