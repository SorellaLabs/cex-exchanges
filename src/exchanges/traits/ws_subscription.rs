use super::SpecificWsChannel;

pub trait SpecificWsSubscription {
    /// enum of channel types for this exchange
    type WsChannel: SpecificWsChannel;
    /// enum of channel types for this exchange
    type TradingPair;

    /// adds a channel with subscription parameters
    fn add_channel(&mut self, channel: Self::WsChannel);

    /// removes a pair from the current subscription, returns true if the
    /// subscription is currently empty
    fn remove_pair(&mut self, pair: &Self::TradingPair) -> bool;
}
