use super::{
    channels::{KucoinWsChannel, KucoinWsChannelKind},
    KucoinMultiSubscription,
};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MultiWsStreamBuilder},
    kucoin::Kucoin,
    normalized::ws::NormalizedWsChannels,
    traits::SpecificWsBuilder,
    CexExchange,
};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
const MAX_KUCOIN_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 100 streams.
pub const MAX_KUCOIN_WS_CONNS_PER_STREAM: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct KucoinWsBuilder {
    pub channels: Vec<KucoinWsChannel>,
}

impl KucoinWsBuilder {
    async fn build_from_all_instruments_util(channels: &[KucoinWsChannelKind], streams_per_connection: Option<usize>) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Kucoin>()
            .await?
            .take_kucoin_instruments(true)
            .unwrap();
        all_symbols_vec.retain(|sym| sym.enable_trading);

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.symbol)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(streams_per_connection.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION));

        chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    KucoinWsChannelKind::Match => KucoinWsChannel::Match(chk.to_vec()),
                    KucoinWsChannelKind::Ticker => KucoinWsChannel::Ticker(chk.to_vec()),
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        });

        Ok(this)
    }
}

impl SpecificWsBuilder for KucoinWsBuilder {
    type CexExchange = Kucoin;
    type WsChannel = KucoinWsChannel;

    /// There is a limit of 300 connections per attempt every 5 minutes per IP.
    const MAX_CONNECTIONS: usize = 300;
    /// A single connection can listen to a maximum of 100 streams.
    const MAX_STREAMS_PER_CONNECTION: usize = 100;

    fn add_channel(mut self, channel: Self::WsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    fn build_single(self) -> Self::CexExchange {
        let mut subscription = KucoinMultiSubscription::default();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Kucoin::new_ws_subscription(subscription)
    }

    fn build_many_distributed(self) -> eyre::Result<MultiWsStreamBuilder<Self::CexExchange>> {
        let stream_size = if self.channels.len() <= MAX_KUCOIN_STREAMS { 1 } else { self.channels.len() / MAX_KUCOIN_STREAMS };

        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = KucoinMultiSubscription::default();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Kucoin::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MultiWsStreamBuilder::new(split_exchange))
    }

    fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MultiWsStreamBuilder<Self::CexExchange>> {
        let chunks = self
            .channels
            .chunks(connections_per_stream.unwrap_or(MAX_KUCOIN_WS_CONNS_PER_STREAM))
            .collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = KucoinMultiSubscription::default();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Kucoin::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MultiWsStreamBuilder::new(split_exchange))
    }

    async fn build_from_all_instruments<'a>(
        channels: &'a [<Self::WsChannel as crate::traits::SpecificWsChannel>::ChannelKind],
        streams_per_connection: Option<usize>,
        _: Option<CexExchange>,
    ) -> eyre::Result<MultiWsStreamBuilder<Self::CexExchange>> {
        let this = Self::build_from_all_instruments_util(channels, streams_per_connection).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = KucoinMultiSubscription::default();
                subscription.add_channel(ch);

                Kucoin::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MultiWsStreamBuilder::new(all_streams))
    }

    fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, _: Option<CexExchange>) -> eyre::Result<Self>
    where
        Self: Sized,
    {
        let mut this = Self { channels: Vec::new() };

        map.into_iter().try_for_each(|channel| {
            let this_channel: KucoinWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
