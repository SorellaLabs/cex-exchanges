use super::pairs::NormalizedTradingPair;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannels {
    Status,
    Trades(Option<Vec<NormalizedTradingPair>>),
}

impl NormalizedWsChannels {
    pub fn new_default(kind: NormalizedWsChannelKinds) -> Self {
        match kind {
            NormalizedWsChannelKinds::Status => NormalizedWsChannels::Status,
            NormalizedWsChannelKinds::Trades => NormalizedWsChannels::Trades(None),
        }
    }

    pub fn new_with_pairs(
        kind: NormalizedWsChannelKinds,
        symbols: &[String],
        delimiter: char,
    ) -> Self {
        match kind {
            NormalizedWsChannelKinds::Status => NormalizedWsChannels::Status,
            NormalizedWsChannelKinds::Trades => {
                NormalizedWsChannels::new_trades_with_pairs(symbols, delimiter)
            }
        }
    }

    fn new_trades_with_pairs(symbols: &[String], delimiter: char) -> NormalizedWsChannels {
        let split_pairs = symbols
            .iter()
            .map(|s| {
                let mut symbols = s.split(delimiter);
                NormalizedTradingPair {
                    base: symbols.next().unwrap().to_string(),
                    quote: symbols.next().unwrap().to_string(),
                }
            })
            .collect();
        NormalizedWsChannels::Trades(Some(split_pairs))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NormalizedWsChannelKinds {
    Status,
    Trades,
}

impl From<NormalizedWsChannels> for NormalizedWsChannelKinds {
    fn from(value: NormalizedWsChannels) -> Self {
        match value {
            NormalizedWsChannels::Status => NormalizedWsChannelKinds::Status,
            NormalizedWsChannels::Trades(_) => NormalizedWsChannelKinds::Trades,
        }
    }
}
