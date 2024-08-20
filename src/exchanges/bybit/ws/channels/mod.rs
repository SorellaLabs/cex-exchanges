mod orderbook;
pub use orderbook::*;

mod trades;
use std::fmt::Display;

pub use trades::*;

use crate::{
    exchanges::{
        bybit::pairs::BybitTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    traits::SpecificWsChannel,
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BybitWsChannel {
    Trade(Vec<BybitTradingPair>),
    OrderbookL1(Vec<BybitTradingPair>)
}

impl SpecificWsChannel for BybitWsChannel {
    type ChannelKind = BybitWsChannelKind;

    fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Bybit))
            .collect();

        Self::new_from_normalized(BybitWsChannel::Trade(Vec::new()), normalized)
    }

    fn new_quote(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Bybit))
            .collect();

        Self::new_from_normalized(BybitWsChannel::OrderbookL1(Vec::new()), normalized)
    }

    fn new_l2(_: Option<u64>, _: u64, _: Vec<RawTradingPair>) -> eyre::Result<Self> {
        unreachable!()
    }

    fn new_from_normalized(self, pairs: Vec<NormalizedTradingPair>) -> eyre::Result<Self> {
        match self {
            BybitWsChannel::Trade(_) => Ok(BybitWsChannel::Trade(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            BybitWsChannel::OrderbookL1(_) => Ok(BybitWsChannel::OrderbookL1(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }

    fn count_entries(&self) -> usize {
        match self {
            BybitWsChannel::Trade(vals) => vals.len(),
            BybitWsChannel::OrderbookL1(vals) => vals.len()
        }
    }
}

impl Display for BybitWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BybitWsChannel::Trade(_) => write!(f, "trade"),
            BybitWsChannel::OrderbookL1(_) => write!(f, "orderbook.1")
        }
    }
}

impl TryFrom<String> for BybitWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" | "publicTrade" => Ok(Self::Trade(Vec::new())),
            "orderbook.1" | "quote" => Ok(Self::OrderbookL1(Vec::new())),
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

            NormalizedWsChannels::Quotes(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(BybitWsChannel::OrderbookL1(norm_pairs))
            }
            NormalizedWsChannels::L2(..) => unimplemented!("L2 is not implemented for Bybit")
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BybitWsChannelKind {
    Trade,
    OrderbookL1
}

impl Display for BybitWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BybitWsChannelKind::Trade => write!(f, "publicTrade"),
            BybitWsChannelKind::OrderbookL1 => write!(f, "orderbook.1")
        }
    }
}

impl From<&BybitWsChannel> for BybitWsChannelKind {
    fn from(value: &BybitWsChannel) -> Self {
        match value {
            BybitWsChannel::Trade(_) => BybitWsChannelKind::Trade,
            BybitWsChannel::OrderbookL1(_) => BybitWsChannelKind::OrderbookL1
        }
    }
}
