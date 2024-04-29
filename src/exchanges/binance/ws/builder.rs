use super::{BinanceSubscription, BinanceWsChannel, BinanceWsChannelKind};
use crate::{
    binance::Binance,
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    normalized::ws::NormalizedWsChannels
};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
/// (https://binance-docs.github.io/apidocs/spot/en/#limits)
const MAX_BINANCE_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 1024 streams.
/// (https://binance-docs.github.io/apidocs/spot/en/#limits)
const MAX_BINANCE_WS_CONNS_PER_STREAM: usize = 1024;

#[derive(Debug, Clone, Default)]
pub struct BinanceWsBuilder {
    pub channels: Vec<BinanceWsChannel>
}

impl BinanceWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: BinanceWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// builds a single ws instance of [Binance], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Binance {
        let mut subscription = BinanceSubscription::new();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Binance::new_ws_subscription(subscription)
    }

    /// builds many ws instances of the [Binance] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each with size # channels / `MAX_BINANCE_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Binance>> {
        let stream_size = if self.channels.len() <= MAX_BINANCE_STREAMS { 1 } else { self.channels.len() / MAX_BINANCE_STREAMS };
        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = BinanceSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Binance::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds many ws instances of the [Binance] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Binance>> {
        let chunks = self
            .channels
            .chunks(MAX_BINANCE_WS_CONNS_PER_STREAM)
            .collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = BinanceSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Binance::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds a mutlistream channel from all active instruments
    pub async fn build_from_all_instruments(channels: &[BinanceWsChannelKind]) -> eyre::Result<MutliWsStreamBuilder<Binance>> {
        let this = Self::build_from_all_instruments_util(channels).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = BinanceSubscription::new();
                subscription.add_channel(ch);

                Binance::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_from_all_instruments_util(channels: &[BinanceWsChannelKind]) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Binance>()
            .await?
            .take_binance_instruments()
            .unwrap();
        all_symbols_vec.retain(|sym| &sym.status == "TRADING");

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.symbol)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(MAX_BINANCE_WS_CONNS_PER_STREAM);

        chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    BinanceWsChannelKind::Trade => BinanceWsChannel::Trade(chk.to_vec()),
                    BinanceWsChannelKind::BookTicker => BinanceWsChannel::BookTicker(chk.to_vec())
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
            let this_channel: BinanceWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
