use std::fmt::Display;

mod matches;
pub use matches::*;

mod ticker;
pub use ticker::*;

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
    Match(Vec<KucoinTradingPair>),
    Ticker(Vec<KucoinTradingPair>)
}

impl KucoinWsChannel {
    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_match(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Kucoin))
            .collect();

        Self::new_from_normalized(normalized, KucoinWsChannel::Match(Vec::new()))
    }

    /// builds the book ticker channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    pub fn new_ticker(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Kucoin))
            .collect();

        Self::new_from_normalized(normalized, KucoinWsChannel::Ticker(Vec::new()))
    }

    pub(crate) fn new_from_normalized(pairs: Vec<NormalizedTradingPair>, kind: KucoinWsChannel) -> eyre::Result<Self> {
        match kind {
            KucoinWsChannel::Match(_) => Ok(KucoinWsChannel::Match(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            KucoinWsChannel::Ticker(_) => Ok(KucoinWsChannel::Ticker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }

    pub fn count_entries(&self) -> usize {
        match self {
            KucoinWsChannel::Match(vals) => vals.len(),
            KucoinWsChannel::Ticker(vals) => vals.len()
        }
    }
}

impl Display for KucoinWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KucoinWsChannel::Match(_) => write!(f, "match"),
            KucoinWsChannel::Ticker(_) => write!(f, "ticker")
        }
    }
}

impl TryFrom<String> for KucoinWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "match" => Ok(Self::Match(Vec::new())),
            "ticker" => Ok(Self::Ticker(Vec::new())),
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

                Ok(KucoinWsChannel::Match(norm_pairs))
            }
            NormalizedWsChannels::Quotes(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(KucoinWsChannel::Ticker(norm_pairs))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KucoinWsChannelKind {
    Match,
    Ticker
}

impl Display for KucoinWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KucoinWsChannelKind::Match => write!(f, "match"),
            KucoinWsChannelKind::Ticker => write!(f, "ticker")
        }
    }
}

impl From<&KucoinWsChannel> for KucoinWsChannelKind {
    fn from(value: &KucoinWsChannel) -> Self {
        match value {
            KucoinWsChannel::Match(_) => KucoinWsChannelKind::Match,
            KucoinWsChannel::Ticker(_) => KucoinWsChannelKind::Ticker
        }
    }
}
