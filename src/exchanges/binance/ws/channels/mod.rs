mod book_ticker;
pub use book_ticker::*;

mod diff_depth;
pub use diff_depth::*;

mod trades;
use std::fmt::Display;

pub use trades::*;

use crate::{
    exchanges::{
        binance::pairs::BinanceTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    traits::SpecificWsChannel,
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BinanceWsChannel {
    Trade(Vec<BinanceTradingPair>),
    BookTicker(Vec<BinanceTradingPair>),
    /// (depth (5, 10, or 20), update speed (100ms or 1000ms), trading pairs)
    DiffDepth(Option<u64>, u64, Vec<BinanceTradingPair>)
}

impl SpecificWsChannel for BinanceWsChannel {
    type ChannelKind = BinanceWsChannelKind;

    fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect();

        Self::new_from_normalized(BinanceWsChannel::Trade(Vec::new()), normalized)
    }

    fn new_quote(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect();

        Self::new_from_normalized(BinanceWsChannel::BookTicker(Vec::new()), normalized)
    }

    fn new_l2(depth: Option<u64>, update_speed: u64, pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        if depth.is_some() {
            unimplemented!("depth is not implemented for Binance yet")
        }
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect();

        Self::new_from_normalized(BinanceWsChannel::DiffDepth(depth, update_speed, Vec::new()), normalized)
    }

    fn new_from_normalized(self, pairs: Vec<NormalizedTradingPair>) -> eyre::Result<Self> {
        match self {
            BinanceWsChannel::Trade(_) => Ok(BinanceWsChannel::Trade(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            BinanceWsChannel::BookTicker(_) => Ok(BinanceWsChannel::BookTicker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            BinanceWsChannel::DiffDepth(depth, update_speed, _) => Ok(BinanceWsChannel::DiffDepth(
                depth,
                update_speed,
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }

    fn count_entries(&self) -> usize {
        match self {
            BinanceWsChannel::Trade(vals) => vals.len(),
            BinanceWsChannel::BookTicker(vals) => vals.len(),
            BinanceWsChannel::DiffDepth(_, _, vals) => vals.len()
        }
    }
}

impl Display for BinanceWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannel::Trade(_) => write!(f, "trade"),
            BinanceWsChannel::BookTicker(_) => write!(f, "bookTicker"),
            BinanceWsChannel::DiffDepth(depth, update_speed, _) => {
                if let Some(d) = depth {
                    write!(f, "depth{d}@{update_speed}ms")
                } else {
                    write!(f, "depth@{update_speed}ms")
                }
            }
        }
    }
}

impl TryFrom<String> for BinanceWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" => Ok(Self::Trade(Vec::new())),
            "bookticker" => Ok(Self::BookTicker(Vec::new())),
            "depth@100ms" => Ok(Self::DiffDepth(None, 100, Vec::new())),
            "depth@1000ms" => Ok(Self::DiffDepth(None, 1000, Vec::new())),
            "depth5@100ms" => Ok(Self::DiffDepth(Some(5), 100, Vec::new())),
            "depth5@1000ms" => Ok(Self::DiffDepth(Some(5), 1000, Vec::new())),
            "depth10@100ms" => Ok(Self::DiffDepth(Some(10), 100, Vec::new())),
            "depth10@1000ms" => Ok(Self::DiffDepth(Some(10), 1000, Vec::new())),
            "depth20@100ms" => Ok(Self::DiffDepth(Some(20), 100, Vec::new())),
            "depth20@1000ms" => Ok(Self::DiffDepth(Some(20), 1000, Vec::new())),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl TryFrom<NormalizedWsChannels> for BinanceWsChannel {
    type Error = eyre::ErrReport;

    fn try_from(value: NormalizedWsChannels) -> Result<Self, Self::Error> {
        match value {
            NormalizedWsChannels::Trades(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(BinanceWsChannel::Trade(norm_pairs))
            }
            NormalizedWsChannels::Quotes(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(BinanceWsChannel::BookTicker(norm_pairs))
            }

            NormalizedWsChannels::L2(depth, update_speed, pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                if depth.is_some() {
                    unimplemented!("depth is not implemented for Binance L2 yet")
                }

                Ok(BinanceWsChannel::DiffDepth(depth, update_speed, norm_pairs))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinanceWsChannelKind {
    Trade,
    BookTicker,
    DiffDepth(Option<u64>, u64)
}

impl Display for BinanceWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannelKind::Trade => write!(f, "trade"),
            BinanceWsChannelKind::BookTicker => write!(f, "bookTicker"),
            BinanceWsChannelKind::DiffDepth(depth, update_speed) => {
                if let Some(d) = depth {
                    write!(f, "depth{d}@{update_speed}ms")
                } else {
                    write!(f, "depth@{update_speed}ms")
                }
            }
        }
    }
}

impl From<&BinanceWsChannel> for BinanceWsChannelKind {
    fn from(value: &BinanceWsChannel) -> Self {
        match value {
            BinanceWsChannel::Trade(_) => BinanceWsChannelKind::Trade,
            BinanceWsChannel::BookTicker(_) => BinanceWsChannelKind::BookTicker,
            BinanceWsChannel::DiffDepth(depth, update_speed, _) => BinanceWsChannelKind::DiffDepth(*depth, *update_speed)
        }
    }
}
