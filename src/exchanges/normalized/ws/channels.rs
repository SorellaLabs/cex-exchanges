use clap::ValueEnum;

use crate::{
    exchanges::normalized::types::{NormalizedTradingPair, RawTradingPair},
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannels {
    Trades(Vec<NormalizedTradingPair>),
    Quotes(Vec<NormalizedTradingPair>)
}

impl NormalizedWsChannels {
    pub fn new_default(kind: NormalizedWsChannelKinds) -> Self {
        match kind {
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(Vec::new()),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(Vec::new())
        }
    }

    /// uses specific pairs
    pub fn new_with_pairs(exchange: CexExchange, kind: NormalizedWsChannelKinds, pairs: &[RawTradingPair]) -> Self {
        match kind {
            NormalizedWsChannelKinds::Trades => Self::new_with_pairs_util(exchange, NormalizedWsChannelKinds::Trades, pairs),
            NormalizedWsChannelKinds::Quotes => Self::new_with_pairs_util(exchange, NormalizedWsChannelKinds::Quotes, pairs)
        }
    }

    pub(crate) fn make_many_single(self) -> Vec<Self> {
        match self {
            NormalizedWsChannels::Trades(vals) => vals
                .into_iter()
                .map(|v| NormalizedWsChannels::Trades(vec![v]))
                .collect(),
            NormalizedWsChannels::Quotes(vals) => vals
                .into_iter()
                .map(|v| NormalizedWsChannels::Quotes(vec![v]))
                .collect()
        }
    }

    /// util
    fn new_with_pairs_util(exchange: CexExchange, kind: NormalizedWsChannelKinds, pairs: &[RawTradingPair]) -> NormalizedWsChannels {
        let split_pairs = pairs
            .iter()
            .map(|p| p.get_normalized_pair(exchange))
            .collect();

        match kind {
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(split_pairs),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(split_pairs)
        }
    }

    pub fn add_pairs(&mut self, exchange: CexExchange, pairs: &[RawTradingPair]) {
        let split_pairs = pairs
            .iter()
            .map(|p| p.get_normalized_pair(exchange))
            .collect::<Vec<_>>();

        match self {
            NormalizedWsChannels::Trades(p) => p.extend(split_pairs),
            NormalizedWsChannels::Quotes(p) => p.extend(split_pairs)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum NormalizedWsChannelKinds {
    Trades,
    Quotes
}

impl From<NormalizedWsChannels> for NormalizedWsChannelKinds {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Trades(_) => NormalizedWsChannelKinds::Trades,
            NormalizedWsChannels::Quotes(_) => NormalizedWsChannelKinds::Quotes
        }
    }
}
