mod book_ticker;
pub use book_ticker::*;

mod diff_depth;
pub use diff_depth::*;

mod trades;
use std::fmt::Display;

mod partial_book_depth;
pub use partial_book_depth::*;
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
    /// (depth levels (5, 10, or 20), update speed (100ms or 1000ms), trading
    /// pairs)
    PartialBookDepth(u64, Option<u64>, Vec<BinanceTradingPair>),
    /// (update speed (100ms or 1000ms), trading pairs)
    DiffDepth(Option<u64>, Vec<BinanceTradingPair>)
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

    fn new_l2(depth_levels: Option<u64>, update_speed: Option<u64>, pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect::<Vec<_>>();

        if let Some(levels) = depth_levels {
            Self::new_from_normalized(BinanceWsChannel::PartialBookDepth(levels, update_speed, Vec::new()), normalized)
        } else {
            Self::new_from_normalized(BinanceWsChannel::DiffDepth(update_speed, Vec::new()), normalized)
        }
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
            BinanceWsChannel::PartialBookDepth(depth, update_speed, pairs) => Ok(BinanceWsChannel::PartialBookDepth(
                depth,
                update_speed,
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),

            BinanceWsChannel::DiffDepth(update_speed, pairs) => Ok(BinanceWsChannel::DiffDepth(
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
            BinanceWsChannel::PartialBookDepth(_, _, vals) => vals.len(),
            BinanceWsChannel::DiffDepth(_, vals) => vals.len()
        }
    }
}

impl Display for BinanceWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannel::Trade(_) => write!(f, "trade"),
            BinanceWsChannel::BookTicker(_) => write!(f, "bookTicker"),
            BinanceWsChannel::PartialBookDepth(depth, update_speed, _) => {
                if let Some(s) = update_speed {
                    write!(f, "depth{depth}@{s}ms")
                } else {
                    write!(f, "depth{depth}")
                }
            }

            BinanceWsChannel::DiffDepth(update_speed, _) => {
                if let Some(s) = update_speed {
                    write!(f, "depth@{s}ms")
                } else {
                    write!(f, "depth")
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
            "depth" => Ok(Self::DiffDepth(None, Vec::new())),
            "depth@100ms" => Ok(Self::DiffDepth(Some(1000), Vec::new())),
            "depth@1000ms" => Ok(Self::DiffDepth(Some(1000), Vec::new())),
            "depth5" => Ok(Self::PartialBookDepth(5, None, Vec::new())),
            "depth5@100ms" => Ok(Self::PartialBookDepth(5, Some(100), Vec::new())),
            "depth5@1000ms" => Ok(Self::PartialBookDepth(5, Some(1000), Vec::new())),
            "depth10" => Ok(Self::PartialBookDepth(10, None, Vec::new())),
            "depth10@100ms" => Ok(Self::PartialBookDepth(10, Some(100), Vec::new())),
            "depth10@1000ms" => Ok(Self::PartialBookDepth(10, Some(1000), Vec::new())),
            "depth20" => Ok(Self::PartialBookDepth(20, None, Vec::new())),
            "depth20@100ms" => Ok(Self::PartialBookDepth(20, Some(100), Vec::new())),
            "depth20@1000ms" => Ok(Self::PartialBookDepth(20, Some(1000), Vec::new())),
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

                if let Some(d) = depth {
                    Ok(BinanceWsChannel::PartialBookDepth(d, update_speed, norm_pairs))
                } else {
                    Ok(BinanceWsChannel::DiffDepth(update_speed, norm_pairs))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinanceWsChannelKind {
    Trade,
    BookTicker,
    PartialBookDepth(u64, Option<u64>),
    DiffDepth(Option<u64>)
}

impl Display for BinanceWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannelKind::Trade => write!(f, "trade"),
            BinanceWsChannelKind::BookTicker => write!(f, "bookTicker"),
            BinanceWsChannelKind::PartialBookDepth(depth, update_speed) => {
                if let Some(s) = update_speed {
                    write!(f, "depth{depth}@{s}ms")
                } else {
                    write!(f, "depth{depth}@")
                }
            }
            BinanceWsChannelKind::DiffDepth(update_speed) => {
                if let Some(s) = update_speed {
                    write!(f, "depth@{s}ms")
                } else {
                    write!(f, "depth")
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
            BinanceWsChannel::PartialBookDepth(depth, update_speed, _) => BinanceWsChannelKind::PartialBookDepth(*depth, *update_speed),
            BinanceWsChannel::DiffDepth(update_speed, _) => BinanceWsChannelKind::DiffDepth(*update_speed)
        }
    }
}
