use std::collections::HashMap;

use super::CexExchange;
use crate::types::normalized::ws::channels::{NormalizedWsChannelKinds, NormalizedWsChannels};

#[derive(Debug, Default, Clone)]
pub struct NormalizedExchangeBuilder {
    ws_exchanges: HashMap<CexExchange, HashMap<NormalizedWsChannelKinds, NormalizedWsChannels>>
}

impl NormalizedExchangeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// adds trading pairs to all channels (if applicable)
    pub fn add_pairs_all_channels<C, S>(&mut self, exchange: &CexExchange, channels: &[C], pairs: &[S])
    where
        for<'a> &'a C: Into<NormalizedWsChannelKinds>,
        for<'a> &'a S: Into<TradingPairSplitRaw>
    {
        let entry = self.ws_exchanges.entry(*exchange).or_default();

        channels.iter().for_each(|c| {
            let channel_kind: NormalizedWsChannelKinds = c.into();
            entry
                .entry(channel_kind)
                .or_insert_with(|| NormalizedWsChannels::new_with_pairs(exchange, channel_kind, pairs));
        });
    }

    /// adds trading pairs to a channel (if applicable)
    pub fn add_pairs_channel<C, S>(&mut self, exchange: &CexExchange, channel: &C, pairs: &[S])
    where
        for<'a> &'a C: Into<NormalizedWsChannelKinds>,
        for<'a> &'a S: Into<TradingPairSplitRaw>
    {
        let entry = self.ws_exchanges.entry(*exchange).or_default();

        let channel_kind: NormalizedWsChannelKinds = channel.into();
        entry
            .entry(channel_kind)
            .or_insert_with(|| NormalizedWsChannels::new_with_pairs(exchange, channel_kind, pairs));
    }
}

#[derive(Debug, Clone)]
pub enum TradingPairSplitRaw {
    Split { base: String, quote: String },
    Raw { pair: String, delimiter: char }
}

impl TradingPairSplitRaw {
    pub fn get_base_quote(&self) -> (String, String, Option<char>) {
        let this = self.clone();
        match this {
            TradingPairSplitRaw::Split { base, quote } => (base, quote, None),
            TradingPairSplitRaw::Raw { pair, delimiter } => {
                let mut split = pair.split(delimiter);
                (split.next().unwrap().to_string(), split.next().unwrap().to_string(), Some(delimiter))
            }
        }
    }
}

/// (base, quote)
/// ex: (ETH, USDC)
impl From<(String, String)> for TradingPairSplitRaw {
    fn from(value: (String, String)) -> Self {
        TradingPairSplitRaw::Split { base: value.0.to_uppercase(), quote: value.1.to_uppercase() }
    }
}

/// (base + quote, delimiter)
/// ex: (ETH_USDC, _)
impl From<(String, char)> for TradingPairSplitRaw {
    fn from(value: (String, char)) -> Self {
        if value.1 == '\0' {
            panic!("delimiter for coinbase cannot be empty/null")
        }
        TradingPairSplitRaw::Raw { pair: value.0.to_uppercase(), delimiter: value.1 }
    }
}
