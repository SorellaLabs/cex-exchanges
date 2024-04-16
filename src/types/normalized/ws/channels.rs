use crate::{
    exchanges::{builder::TradingPairSplitRaw, CexExchange},
    types::normalized::pairs::NormalizedTradingPair
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannels {
    Status,
    Trades(Option<Vec<NormalizedTradingPair>>),
    Quotes(Option<Vec<NormalizedTradingPair>>)
}

impl NormalizedWsChannels {
    pub fn new_default(kind: NormalizedWsChannelKinds) -> Self {
        match kind {
            NormalizedWsChannelKinds::Status => NormalizedWsChannels::Status,
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(None),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(None)
        }
    }

    pub fn new_with_pairs<S>(exchange: &CexExchange, kind: NormalizedWsChannelKinds, pairs: &[S]) -> Self
    where
        for<'a> &'a S: Into<TradingPairSplitRaw>
    {
        let pairs: Vec<TradingPairSplitRaw> = pairs.iter().map(Into::into).collect::<Vec<_>>();
        match kind {
            NormalizedWsChannelKinds::Status => NormalizedWsChannels::Status,
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::new_any_with_pairs(exchange, NormalizedWsChannelKinds::Trades, pairs),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::new_any_with_pairs(exchange, NormalizedWsChannelKinds::Quotes, pairs)
        }
    }

    fn new_any_with_pairs(exchange: &CexExchange, kind: NormalizedWsChannelKinds, pairs: Vec<TradingPairSplitRaw>) -> NormalizedWsChannels {
        let split_pairs = pairs
            .iter()
            .map(|p| {
                let (base, quote, delimiter) = p.get_base_quote();
                NormalizedTradingPair::new(*exchange, &base, &quote, delimiter)
            })
            .collect();

        match kind {
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(Some(split_pairs)),
            NormalizedWsChannelKinds::Quotes => NormalizedWsChannels::Quotes(Some(split_pairs)),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannelKinds {
    Status,
    Trades,
    Quotes
}

impl From<NormalizedWsChannels> for NormalizedWsChannelKinds {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Status => NormalizedWsChannelKinds::Status,
            NormalizedWsChannels::Trades(_) => NormalizedWsChannelKinds::Trades,
            NormalizedWsChannels::Quotes(_) => NormalizedWsChannelKinds::Quotes
        }
    }
}
