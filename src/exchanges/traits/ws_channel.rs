use crate::normalized::types::{NormalizedTradingPair, RawTradingPair};

pub trait SpecificWsChannel: Sized {
    /// enum of channel types for this exchange
    type ChannelKind;

    /// builds trade channel from a vec of raw trading pairs
    /// return an error if the symbol is incorrectly formatted
    fn new_trade(pairs: Vec<RawTradingPair>) -> eyre::Result<Self>;

    /// builds the quote channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    fn new_quote(pairs: Vec<RawTradingPair>) -> eyre::Result<Self>;

    /// builds the quote channel from a vec of raw trading
    /// pairs return an error if the symbol is incorrectly formatted
    fn new_l2(depth: Option<u64>, update_speed: Option<u64>, pairs: Vec<RawTradingPair>) -> eyre::Result<Self>;

    fn new_from_normalized(self, pairs: Vec<NormalizedTradingPair>) -> eyre::Result<Self>;

    // return the number of entries in the channel
    fn count_entries(&self) -> usize;

    // channel kind (without values)
    fn kind(&self) -> Self::ChannelKind
    where
        for<'a> &'a Self: Into<Self::ChannelKind>
    {
        self.into()
    }
}
