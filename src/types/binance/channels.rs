use std::fmt::Display;

use super::pairs::BinanceTradingPair;
use crate::types::normalized::ws::channels::NormalizedWsChannels;

#[derive(Debug, Clone)]
pub enum BinanceChannel {
    Trade(Option<Vec<BinanceTradingPair>>)
}

impl BinanceChannel {
    /// builds ticker channel from a vec of (base asset, quote asset)
    /// (eth, usdt) -> ETHUSDT
    pub fn new_trade_from_base_quote(pairs: Vec<(String, String)>) -> Self {
        BinanceChannel::Trade(Some(
            pairs
                .into_iter()
                .map(|(b, q)| BinanceTradingPair::new_unchecked(&format!("{}{}", b.to_uppercase(), q.to_uppercase())))
                .collect()
        ))
    }

    /// builds trade channel from a vec of trading pairs
    /// eth_USDT -> ETHUSDT
    /// panics if the symbol is incorrectly formatted
    pub fn new_trade_from_pair(pairs: Vec<String>) -> Self {
        BinanceChannel::Trade(Some(
            pairs
                .into_iter()
                .map(|s| {
                    BinanceTradingPair::new_unchecked(
                        &s.replace("-", "")
                            .replace("_", "")
                            .replace("/", "")
                            .to_uppercase()
                    )
                })
                .collect()
        ))
    }

    pub(crate) fn build_url(&self) -> String {
        match self {
            BinanceChannel::Trade(Some(vals)) => vals
                .into_iter()
                .map(|val| format!("{}@trade", val.0.to_lowercase()))
                .collect::<Vec<_>>()
                .join("/"),
            _ => unreachable!()
        }
    }
}

impl Display for BinanceChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinanceChannel::Trade(_) => write!(f, "trade")
        }
    }
}

impl TryFrom<String> for BinanceChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "trade" => Ok(Self::Trade(None)),
            _ => Err(eyre::ErrReport::msg(format!("channel is not valid: {value}")))
        }
    }
}

impl From<NormalizedWsChannels> for BinanceChannel {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Trades(pairs) => BinanceChannel::Trade(pairs.map(|p| p.into_iter().map(Into::into).collect())),
            _ => unimplemented!()
        }
    }
}
