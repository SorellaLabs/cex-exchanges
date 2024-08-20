use std::fmt::Display;

mod matches;
pub use matches::*;

mod ticker;
pub use ticker::*;

mod status;
pub use status::*;

use crate::{
    exchanges::{
        coinbase::pairs::CoinbaseTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    traits::SpecificWsChannel,
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CoinbaseWsChannel {
    Status,
    Matches(Vec<CoinbaseTradingPair>),
    Ticker(Vec<CoinbaseTradingPair>)
}

impl SpecificWsChannel for CoinbaseWsChannel {
    type ChannelKind = CoinbaseWsChannelKind;

    fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Coinbase))
            .collect();

        Self::new_from_normalized(CoinbaseWsChannel::Matches(Vec::new()), normalized)
    }

    fn new_quote(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Coinbase))
            .collect();

        Self::new_from_normalized(CoinbaseWsChannel::Ticker(Vec::new()), normalized)
    }

    fn new_l2(_: Option<u64>, _: u64, _: Vec<RawTradingPair>) -> eyre::Result<Self> {
        unreachable!()
    }

    fn new_from_normalized(self, pairs: Vec<NormalizedTradingPair>) -> eyre::Result<Self> {
        match self {
            CoinbaseWsChannel::Matches(_) => Ok(CoinbaseWsChannel::Matches(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            CoinbaseWsChannel::Ticker(_) => Ok(CoinbaseWsChannel::Ticker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            CoinbaseWsChannel::Status => Ok(CoinbaseWsChannel::Status)
        }
    }

    fn count_entries(&self) -> usize {
        match self {
            CoinbaseWsChannel::Matches(vals) => vals.len(),
            CoinbaseWsChannel::Ticker(vals) => vals.len(),
            CoinbaseWsChannel::Status => 0
        }
    }
}

impl Display for CoinbaseWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinbaseWsChannel::Status => write!(f, "status"),
            CoinbaseWsChannel::Matches(_) => write!(f, "matches"),
            CoinbaseWsChannel::Ticker(_) => write!(f, "ticker")
        }
    }
}

impl TryFrom<String> for CoinbaseWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "status" => Ok(Self::Status),
            "matches" => Ok(Self::Matches(Vec::new())),
            "ticker" => Ok(Self::Ticker(Vec::new())),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl TryFrom<NormalizedWsChannels> for CoinbaseWsChannel {
    type Error = eyre::ErrReport;

    fn try_from(value: NormalizedWsChannels) -> Result<Self, Self::Error> {
        match value {
            NormalizedWsChannels::Trades(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(CoinbaseWsChannel::Matches(norm_pairs))
            }

            NormalizedWsChannels::Quotes(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(CoinbaseWsChannel::Ticker(norm_pairs))
            }
            NormalizedWsChannels::L2(..) => unimplemented!("L2 is not implemented for Coinbase")
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CoinbaseWsChannelKind {
    Matches,
    Ticker,
    Status
}

impl Display for CoinbaseWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinbaseWsChannelKind::Matches => write!(f, "matches"),
            CoinbaseWsChannelKind::Ticker => write!(f, "ticker"),
            CoinbaseWsChannelKind::Status => write!(f, "status")
        }
    }
}

impl From<&CoinbaseWsChannel> for CoinbaseWsChannelKind {
    fn from(value: &CoinbaseWsChannel) -> Self {
        match value {
            CoinbaseWsChannel::Matches(_) => CoinbaseWsChannelKind::Matches,
            CoinbaseWsChannel::Ticker(_) => CoinbaseWsChannelKind::Ticker,
            CoinbaseWsChannel::Status => CoinbaseWsChannelKind::Status
        }
    }
}
