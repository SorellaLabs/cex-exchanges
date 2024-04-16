use std::collections::HashMap;

use crate::types::normalized::channels::{NormalizedWsChannelKinds, NormalizedWsChannels};

use super::normalized::CexExchange;

#[derive(Debug, Default, Clone)]
pub struct NormalizedExchangeBuilder {
    ws_exchanges: HashMap<CexExchange, HashMap<NormalizedWsChannelKinds, NormalizedWsChannels>>,
}

impl NormalizedExchangeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// adds trading pairs to all channels (if applicable)
    ///
    /// `symbol_delimiter` is how to split the 2 symbols in the pair
    /// (`ETH-USDT` -> symbol_delimiter = `-`)
    pub fn add_pairs_all_channels<C>(
        &mut self,
        exchange: &CexExchange,
        channels: &[C],
        pairs: &[String],
        symbol_delimiter: char,
    ) where
        for<'a> &'a C: Into<NormalizedWsChannelKinds>,
    {
        let entry = self
            .ws_exchanges
            .entry(*exchange)
            .or_default();

        channels.iter().for_each(|c| {
            let channel_kind: NormalizedWsChannelKinds = c.into();
            entry.entry(channel_kind).or_insert_with(|| {
                NormalizedWsChannels::new_with_pairs(channel_kind, pairs, symbol_delimiter)
            });
        });
    }

    /// adds  trading pairs to a channel (if applicable)
    ///
    /// `symbol_delimiter` is how to split the 2 symbols in the pair
    /// (`ETH-USDT` -> symbol_delimiter = `-`)
    pub fn add_pairs_channel<C>(
        &mut self,
        exchange: &CexExchange,
        channel: &C,
        pairs: &[String],
        symbol_delimiter: char,
    ) where
        for<'a> &'a C: Into<NormalizedWsChannelKinds>,
    {
        let entry = self
            .ws_exchanges
            .entry(*exchange)
            .or_default();

        let channel_kind: NormalizedWsChannelKinds = channel.into();
        entry.entry(channel_kind).or_insert_with(|| {
            NormalizedWsChannels::new_with_pairs(channel_kind, pairs, symbol_delimiter)
        });
    }
}
