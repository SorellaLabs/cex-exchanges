use std::fmt::Display;

use serde::Serialize;

use super::pairs::CoinbaseTradingPair;
use crate::types::normalized::ws::channels::NormalizedWsChannels;

#[derive(Debug, Clone)]
pub enum CoinbaseChannel {
    Status,
    Match(Option<Vec<CoinbaseTradingPair>>),
    Ticker(Option<Vec<CoinbaseTradingPair>>)
}

impl CoinbaseChannel {
    /// builds  match channel from a vec of (base asset, quote asset)
    /// (eth, usdt) -> ETH-USDT
    pub fn new_match_from_base_quote(pairs: Vec<(String, String)>) -> Self {
        CoinbaseChannel::Match(Some(
            pairs
                .into_iter()
                .map(|(b, q)| CoinbaseTradingPair::new_unchecked(&format!("{}-{}", b.to_uppercase(), q.to_uppercase())))
                .collect()
        ))
    }

    /// builds  match channel from a vec of trading pairs
    /// eth_USDT -> ETH-USDT
    /// panics if the symbol is incorrectly formatted
    pub fn new_match_from_pair(pairs: Vec<String>, delimiter: char) -> Self {
        if delimiter == '\0' {
            panic!("delimiter for coinbase cannot be empty/null")
        }

        CoinbaseChannel::Match(Some(
            pairs
                .into_iter()
                .map(|s| CoinbaseTradingPair::new_unchecked(&s.replace(delimiter, "-").to_uppercase()))
                .collect()
        ))
    }

    /// builds ticker channel from a vec of (base asset, quote asset)
    /// (eth, usdt) -> ETH-USDT
    pub fn new_ticker_from_base_quote(pairs: Vec<(String, String)>) -> Self {
        CoinbaseChannel::Ticker(Some(
            pairs
                .into_iter()
                .map(|(b, q)| CoinbaseTradingPair::new_unchecked(&format!("{}-{}", b.to_uppercase(), q.to_uppercase())))
                .collect()
        ))
    }

    /// builds ticker channel from a vec of trading pairs
    /// eth_USDT -> ETH-USDT
    /// panics if the symbol is incorrectly formatted
    pub fn new_ticker_from_pair(pairs: Vec<String>, delimiter: char) -> Self {
        if delimiter == '\0' {
            panic!("delimiter for coinbase cannot be empty/null")
        }

        CoinbaseChannel::Ticker(Some(
            pairs
                .into_iter()
                .map(|s| CoinbaseTradingPair::new_unchecked(&s.replace(delimiter, "-").to_uppercase()))
                .collect()
        ))
    }
}

impl Display for CoinbaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinbaseChannel::Status => write!(f, "status"),
            CoinbaseChannel::Match(_) => write!(f, "match"),
            CoinbaseChannel::Ticker(_) => write!(f, "ticker")
        }
    }
}

impl TryFrom<String> for CoinbaseChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "status" => Ok(Self::Status),
            "match" => Ok(Self::Match(None)),
            "ticker" => Ok(Self::Ticker(None)),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl From<NormalizedWsChannels> for CoinbaseChannel {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Status => CoinbaseChannel::Status,
            NormalizedWsChannels::Trades(pairs) => CoinbaseChannel::Match(pairs.map(|p| p.into_iter().map(Into::into).collect())),
            NormalizedWsChannels::Quotes(pairs) => CoinbaseChannel::Ticker(pairs.map(|p| p.into_iter().map(Into::into).collect()))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CoinbaseSubscription {
    #[serde(rename = "type")]
    sub_name: String,
    channels: Vec<CoinbaseSubscriptionInner>
}

impl Default for CoinbaseSubscription {
    fn default() -> Self {
        Self::new()
    }
}

impl CoinbaseSubscription {
    pub fn new() -> Self {
        CoinbaseSubscription { sub_name: "subscribe".to_string(), channels: Vec::new() }
    }

    pub fn new_single_channel(channel: CoinbaseChannel) -> Self {
        CoinbaseSubscription { sub_name: "subscribe".to_string(), channels: vec![channel.into()] }
    }

    pub(crate) fn add_channel(&mut self, channel: CoinbaseChannel) {
        self.channels.push(channel.into());
    }
}

#[derive(Debug, Clone, Serialize)]
struct CoinbaseSubscriptionInner {
    name:        String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    product_ids: Vec<CoinbaseTradingPair>
}

impl From<CoinbaseChannel> for CoinbaseSubscriptionInner {
    fn from(value: CoinbaseChannel) -> Self {
        let name = value.to_string();
        match value {
            CoinbaseChannel::Status => CoinbaseSubscriptionInner { name, product_ids: Vec::new() },
            CoinbaseChannel::Match(pairs) => CoinbaseSubscriptionInner { name, product_ids: pairs.unwrap_or_default() },
            CoinbaseChannel::Ticker(pairs) => CoinbaseSubscriptionInner { name, product_ids: pairs.unwrap_or_default() }
        }
    }
}
