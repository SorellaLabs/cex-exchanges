use std::future::Future;

use super::SpecificWsChannel;
use crate::{clients::ws::MultiWsStreamBuilder, normalized::ws::NormalizedWsChannels, CexExchange, Exchange};

pub trait SpecificWsBuilder {
    /// maximum number of connections per IP
    const MAX_CONNECTIONS: usize;
    /// maximum # of streams per connection
    const MAX_STREAMS_PER_CONNECTION: usize;

    /// enum of channel types for this exchange
    type WsChannel: SpecificWsChannel;
    /// exchange for this builder
    type CexExchange: Exchange;

    /// adds a channel to the builder
    fn add_channel(self, channel: Self::WsChannel) -> Self;

    /// builds a single ws instance of [Self::CexExchange], handling all
    /// channels on 1 stream
    fn build_single(self) -> Self::CexExchange;

    /// builds many ws instances of the [Self::CexExchange] as the inner streams
    /// of [MutliWsStreamBuilder], splitting the channels into different
    /// streams, each with size # channels / [Self::MAX_CONNECTIONS]
    ///
    /// WARNING: too many channels may break the stream
    fn build_many_distributed(self) -> eyre::Result<MultiWsStreamBuilder<Self::CexExchange>>;

    /// builds many ws instances of the [Self::CexExchange] as the inner streams
    /// of [MutliWsStreamBuilder], splitting the channels into different
    /// streams, each of size [Self::MAX_STREAMS_PER_CONNECTION]
    fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MultiWsStreamBuilder<Self::CexExchange>>;

    /// builds a mutlistream channel from all active instruments
    ///
    /// `streams_per_connection` default to [Self::MAX_STREAMS_PER_CONNECTION]
    /// if `None`
    ///
    /// `exch_currency_proxy` is the proxy for which to get the symbols (ONLY
    /// APPLICABLE FOR OKEX!!!)
    fn build_from_all_instruments<'a>(
        channels: &'a [<Self::WsChannel as SpecificWsChannel>::ChannelKind],
        streams_per_connection: Option<usize>,
        exch_currency_proxy: Option<CexExchange>,
    ) -> impl Future<Output = eyre::Result<MultiWsStreamBuilder<Self::CexExchange>>> + 'a;

    /// makes the builder from the normalized builder's map
    fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, exch_currency_proxy: Option<CexExchange>) -> eyre::Result<Self>
    where
        Self: Sized;
}
