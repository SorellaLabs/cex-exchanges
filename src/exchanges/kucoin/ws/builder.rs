use super::{KucoinMultiSubscription, KucoinWsChannel, KucoinWsChannelKind};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    kucoin::Kucoin,
    normalized::ws::NormalizedWsChannels
};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
const MAX_KUCOIN_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 1024 streams.
const MAX_KUCOIN_WS_CONNS_PER_STREAM: usize = 1024;

#[derive(Debug, Clone, Default)]
pub struct KucoinWsBuilder {
    pub channels: Vec<KucoinWsChannel>
}

impl KucoinWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: KucoinWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// builds a single ws instance of [Kucoin], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Kucoin {
        let mut subscription = KucoinMultiSubscription::default();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Kucoin::new_ws_subscription(subscription)
    }

    /// builds many ws instances of the [Kucoin] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each with size # channels / `MAX_BINANCE_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Kucoin>> {
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

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds many ws instances of the [Kucoin] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MutliWsStreamBuilder<Kucoin>> {
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

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds a mutlistream channel from all active instruments
    pub async fn build_from_all_instruments(channels: &[KucoinWsChannelKind]) -> eyre::Result<MutliWsStreamBuilder<Kucoin>> {
        let this = Self::build_from_all_instruments_util(channels).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = KucoinMultiSubscription::default();
                subscription.add_channel(ch);

                Kucoin::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_from_all_instruments_util(channels: &[KucoinWsChannelKind]) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Kucoin>()
            .await?
            .take_kucoin_instruments()
            .unwrap();
        all_symbols_vec.retain(|sym| sym.enable_trading);

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.symbol)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(MAX_KUCOIN_WS_CONNS_PER_STREAM);

        chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    KucoinWsChannelKind::Match => KucoinWsChannel::Match(chk.to_vec()),
                    KucoinWsChannelKind::Ticker => KucoinWsChannel::Ticker(chk.to_vec())
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        });

        Ok(this)
    }

    /// makes the builder from the normalized builder's map
    pub(crate) fn make_from_normalized_map(map: Vec<NormalizedWsChannels>) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new() };

        map.into_iter().try_for_each(|channel| {
            let this_channel: KucoinWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
