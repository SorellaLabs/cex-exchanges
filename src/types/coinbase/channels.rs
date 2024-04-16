use serde::Serialize;

use crate::types::normalized::channels::NormalizedWsChannels;

use super::pairs::CoinbaseTradingPair;

#[derive(Debug, Clone)]
pub enum CoinbaseChannel {
    Status,
    RfqMatch(Option<Vec<CoinbaseTradingPair>>),
}

impl CoinbaseChannel {
    /// builds rfq matches channel from a vec of (base asset, quote asset)
    /// (eth, usdt) -> ETH-USDT
    pub fn new_rfq_matches_from_base_quote(pairs: Vec<(String, String)>) -> Self {
        CoinbaseChannel::RfqMatch(Some(
            pairs
                .into_iter()
                .map(|(b, q)| {
                    CoinbaseTradingPair::new_unchecked(&format!(
                        "{}-{}",
                        b.to_uppercase(),
                        q.to_uppercase()
                    ))
                })
                .collect(),
        ))
    }

    /// builds rfq matches channel from a vec of raw trading pairs
    /// eth_USDT -> ETH-USDT
    /// panics if the symbol is incorrectly formatted
    pub fn new_rfq_matches_from_raw(pairs: Vec<String>, delimiter: char) -> Self {
        if delimiter == '\0' {
            panic!("delimiter for coinbase cannot be empty/null")
        }

        CoinbaseChannel::RfqMatch(Some(
            pairs
                .into_iter()
                .map(|s| {
                    CoinbaseTradingPair::new_unchecked(&s.replace(delimiter, "-").to_uppercase())
                })
                .collect(),
        ))
    }
}

impl ToString for CoinbaseChannel {
    fn to_string(&self) -> String {
        match self {
            CoinbaseChannel::Status => "status".to_string(),
            CoinbaseChannel::RfqMatch(_) => "rfq_matches".to_string(),
        }
    }
}

impl TryFrom<String> for CoinbaseChannel {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "status" => Ok(Self::Status),
            _ => Err(eyre::ErrReport::msg(format!(
                "channel is not valid: {value}"
            ))),
        }
    }
}

impl From<NormalizedWsChannels> for CoinbaseChannel {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Status => CoinbaseChannel::Status,
            NormalizedWsChannels::Trades(pairs) => {
                CoinbaseChannel::RfqMatch(pairs.map(|p| p.into_iter().map(Into::into).collect()))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CoinbaseSubscription {
    #[serde(rename = "type")]
    sub_name: String,
    channels: Vec<CoinbaseSubscriptionInner>,
}

impl CoinbaseSubscription {
    pub fn new() -> Self {
        CoinbaseSubscription {
            sub_name: "subscribe".to_string(),
            channels: Vec::new(),
        }
    }

    pub(crate) fn add_channel(&mut self, channel: CoinbaseSubscriptionInner) {
        self.channels.push(channel);
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoinbaseSubscriptionInner {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    product_ids: Vec<CoinbaseTradingPair>,
}

impl From<CoinbaseChannel> for CoinbaseSubscriptionInner {
    fn from(value: CoinbaseChannel) -> Self {
        let name = value.to_string();
        match value {
            CoinbaseChannel::Status => CoinbaseSubscriptionInner {
                name,
                product_ids: Vec::new(),
            },
            CoinbaseChannel::RfqMatch(pairs) => CoinbaseSubscriptionInner {
                name,
                product_ids: pairs.unwrap_or_default(),
            },
        }
    }
}
