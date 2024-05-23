mod tickers;
pub use tickers::*;

mod trades;
pub use trades::*;

use crate::{
    exchanges::{
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        },
        okex::pairs::OkexTradingPair
    },
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OkexWsChannel {
    TradesAll(Vec<OkexTradingPair>),
    BookTicker(Vec<OkexTradingPair>)
}

impl OkexWsChannel {
    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Okex))
            .collect();

        Self::new_from_kind(normalized, OkexWsChannel::TradesAll(Vec::new()))
    }

    /// builds book ticker channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_book_ticker(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Okex))
            .collect();

        Self::new_from_kind(normalized, OkexWsChannel::BookTicker(Vec::new()))
    }

    fn new_from_kind(pairs: Vec<NormalizedTradingPair>, kind: OkexWsChannel) -> eyre::Result<Self> {
        match kind {
            OkexWsChannel::TradesAll(_) => Ok(OkexWsChannel::TradesAll(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            )),
            OkexWsChannel::BookTicker(_) => Ok(OkexWsChannel::BookTicker(
                pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?
            ))
        }
    }
}

impl std::fmt::Display for OkexWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OkexWsChannel::TradesAll(_) => write!(f, "trades-all"),
            OkexWsChannel::BookTicker(_) => write!(f, "tickers")
        }
    }
}

impl TryFrom<String> for OkexWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trades-all" => Ok(Self::TradesAll(Vec::new())),
            "tickers" => Ok(Self::BookTicker(Vec::new())),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl TryFrom<NormalizedWsChannels> for OkexWsChannel {
    type Error = eyre::ErrReport;

    fn try_from(value: NormalizedWsChannels) -> Result<Self, Self::Error> {
        match value {
            NormalizedWsChannels::Trades(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(OkexWsChannel::TradesAll(norm_pairs))
            }
            NormalizedWsChannels::Quotes(pairs) => {
                let norm_pairs = pairs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(OkexWsChannel::BookTicker(norm_pairs))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OkexWsChannelKind {
    TradesAll,
    BookTicker
}

impl std::fmt::Display for OkexWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OkexWsChannelKind::TradesAll => write!(f, "trades-all"),
            OkexWsChannelKind::BookTicker => write!(f, "bookTicker")
        }
    }
}

impl From<&OkexWsChannel> for OkexWsChannelKind {
    fn from(value: &OkexWsChannel) -> Self {
        match value {
            OkexWsChannel::TradesAll(_) => OkexWsChannelKind::TradesAll,
            OkexWsChannel::BookTicker(_) => OkexWsChannelKind::BookTicker
        }
    }
}
