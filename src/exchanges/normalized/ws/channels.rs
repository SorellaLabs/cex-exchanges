use clap::ValueEnum;

use crate::{
    exchanges::normalized::types::{NormalizedTradingPair, RawTradingPair},
    CexExchange
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannels {
    Trades(Vec<NormalizedTradingPair>),
    Quotes(Vec<NormalizedTradingPair>),
    /// (depth, update speed, trading pairs)
    /// DEFAULT: (None, 1000ms, trading pairs)
    L2(Option<u64>, Option<u64>, Vec<NormalizedTradingPair>)
}

impl NormalizedWsChannels {
    pub fn new_default(kind: NormalizedWsChannelKinds) -> Self {
        match kind {
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(Vec::new()),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(Vec::new()),
            NormalizedWsChannelKinds::L2 => NormalizedWsChannels::L2(None, Some(100), Vec::new())
        }
    }

    /// uses specific pairs
    pub fn new_with_pairs(
        exchange: CexExchange,
        kind: NormalizedWsChannelKinds,
        pairs: &[RawTradingPair],
        l2_config: Option<(Option<u64>, Option<u64>)>
    ) -> Self {
        match kind {
            NormalizedWsChannelKinds::Trades => Self::new_with_pairs_util(exchange, NormalizedWsChannelKinds::Trades, pairs, None),
            NormalizedWsChannelKinds::Quotes => Self::new_with_pairs_util(exchange, NormalizedWsChannelKinds::Quotes, pairs, None),
            NormalizedWsChannelKinds::L2 => Self::new_with_pairs_util(exchange, NormalizedWsChannelKinds::L2, pairs, l2_config)
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
                .collect(),
            NormalizedWsChannels::L2(depth, update_speed, vals) => vals
                .into_iter()
                .map(|v| NormalizedWsChannels::L2(depth, update_speed, vec![v]))
                .collect()
        }
    }

    /// util
    fn new_with_pairs_util(
        exchange: CexExchange,
        kind: NormalizedWsChannelKinds,
        pairs: &[RawTradingPair],
        l2_config: Option<(Option<u64>, Option<u64>)>
    ) -> NormalizedWsChannels {
        let split_pairs = pairs
            .iter()
            .map(|p| p.get_normalized_pair(exchange))
            .collect();

        match kind {
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(split_pairs),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(split_pairs),
            NormalizedWsChannelKinds::L2 => {
                let (depth, update_speed) = l2_config.unwrap_or((None, Some(100)));
                NormalizedWsChannels::L2(depth, update_speed, split_pairs)
            }
        }
    }

    pub fn add_pairs(&mut self, exchange: CexExchange, pairs: &[RawTradingPair]) {
        let split_pairs = pairs
            .iter()
            .map(|p| p.get_normalized_pair(exchange))
            .collect::<Vec<_>>();

        match self {
            NormalizedWsChannels::Trades(p) => p.extend(split_pairs),
            NormalizedWsChannels::Quotes(p) => p.extend(split_pairs),
            NormalizedWsChannels::L2(_, _, p) => p.extend(split_pairs)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, ValueEnum)]
pub enum NormalizedWsChannelKinds {
    Trades,
    Quotes,
    L2
}

impl From<NormalizedWsChannels> for NormalizedWsChannelKinds {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Trades(_) => NormalizedWsChannelKinds::Trades,
            NormalizedWsChannels::Quotes(_) => NormalizedWsChannelKinds::Quotes,
            NormalizedWsChannels::L2(..) => NormalizedWsChannelKinds::L2
        }
    }
}
