use super::{BybitSubscription, BybitWsChannel, BybitWsChannelKind};
use crate::{
    bybit::{Bybit, BybitTradingType},
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    normalized::ws::NormalizedWsChannels
};

/// There is a limit of 500 connections per 5 minutes per IP.
const MAX_BYBIT_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 10 streams.
const MAX_BYBIT_WS_CONNS_PER_STREAM: usize = 10;

#[derive(Debug, Clone, Default)]
pub struct BybitWsBuilder {
    pub channels: Vec<BybitWsChannel>
}

impl BybitWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: BybitWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// splits a [BybitWsChannel] (with values) into mutliple instance of the
    /// same [BybitWsChannel], each with fewer trading pairs by
    /// 'split_channel_size'
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: BybitWsChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            BybitWsChannel::Trade(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| BybitWsChannel::Trade(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
            BybitWsChannel::OrderbookL1(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| BybitWsChannel::OrderbookL1(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
        }

        self
    }

    /// builds a single ws instance of [Bybit], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Bybit {
        let mut subscription = BybitSubscription::new();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Bybit::new_ws_subscription(subscription)
    }

    /// builds many ws instances of the [Bybit] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each with size # channels / `MAX_BYBIT_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Bybit>> {
        let stream_size = if self.channels.len() <= MAX_BYBIT_STREAMS { 1 } else { self.channels.len() / MAX_BYBIT_STREAMS };

        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = BybitSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Bybit::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds many ws instances of the [Bybit] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Bybit>> {
        let chunks = self
            .channels
            .chunks(MAX_BYBIT_WS_CONNS_PER_STREAM)
            .collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = BybitSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Bybit::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds a mutlistream channel from all active instruments
    pub async fn build_from_all_instruments(channels: &[BybitWsChannelKind]) -> eyre::Result<MutliWsStreamBuilder<Bybit>> {
        let this = Self::build_from_all_instruments_util(channels).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = BybitSubscription::new();
                subscription.add_channel(ch);

                Bybit::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_from_all_instruments_util(channels: &[BybitWsChannelKind]) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Bybit>()
            .await?
            .take_bybit_instruments()
            .unwrap();
        all_symbols_vec.retain(|sym| &sym.inner.status == "Trading" && matches!(sym.trading_type, BybitTradingType::Spot));

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.inner.symbol.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let chunks = all_symbols.chunks(MAX_BYBIT_WS_CONNS_PER_STREAM);

        chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    BybitWsChannelKind::Trade => BybitWsChannel::Trade(chk.to_vec()),
                    BybitWsChannelKind::OrderbookL1 => BybitWsChannel::OrderbookL1(chk.to_vec())
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        });

        Ok(this)
    }

    /// makes the builder from the normalized builder's map
    pub(crate) fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, split_channel_size: Option<usize>) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new() };

        map.into_iter().try_for_each(|channel| {
            let this_channel: BybitWsChannel = channel.try_into()?;
            this = if let Some(spl) = split_channel_size {
                this.clone().add_split_channel(this_channel, Some(spl))
            } else {
                this.clone().add_channel(this_channel)
            };
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
