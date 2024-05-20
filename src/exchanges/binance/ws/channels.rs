use std::fmt::Display;

use crate::{
    exchanges::{
        binance::pairs::BinanceTradingPair,
        normalized::{
            types::{NormalizedTradingPair, RawTradingPair},
            ws::NormalizedWsChannels
        }
    },
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BinanceWsChannel {
    Trade(Vec<BinanceTradingPair>),
    BookTicker(Vec<BinanceTradingPair>)
}

impl BinanceWsChannel {
    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    pub fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect();

        Self::new_from_normalized(normalized, BinanceWsChannel::Trade(Vec::new()))
    }

    /// builds the book ticker channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    pub fn new_book_ticker(pairs: Vec<RawTradingPair>) -> eyre::Result<Self> {
        let normalized = pairs
            .into_iter()
            .map(|pair| pair.get_normalized_pair(CexExchange::Binance))
            .collect();

        Self::new_from_normalized(normalized, BinanceWsChannel::BookTicker(Vec::new()))
    }

    pub(crate) fn new_from_normalized(pairs: Vec<NormalizedTradingPair>, kind: BinanceWsChannel) -> eyre::Result<Self> {
        match kind {
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
            ))
        }
    }

    pub fn count_entries(&self) -> usize {
        match self {
            BinanceWsChannel::Trade(vals) => vals.len(),
            BinanceWsChannel::BookTicker(vals) => vals.len()
        }
    }
}

impl Display for BinanceWsChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannel::Trade(_) => write!(f, "trade"),
            BinanceWsChannel::BookTicker(_) => write!(f, "bookTicker")
        }
    }
}

impl TryFrom<String> for BinanceWsChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" => Ok(Self::Trade(Vec::new())),
            "bookticker" => Ok(Self::BookTicker(Vec::new())),
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
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BinanceWsChannelKind {
    Trade,
    BookTicker
}

impl Display for BinanceWsChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceWsChannelKind::Trade => write!(f, "trade"),
            BinanceWsChannelKind::BookTicker => write!(f, "bookTicker")
        }
    }
}

impl From<&BinanceWsChannel> for BinanceWsChannelKind {
    fn from(value: &BinanceWsChannel) -> Self {
        match value {
            BinanceWsChannel::Trade(_) => BinanceWsChannelKind::Trade,
            BinanceWsChannel::BookTicker(_) => BinanceWsChannelKind::BookTicker
        }
    }
}
