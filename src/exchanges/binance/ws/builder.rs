use super::{
    channels::{BinanceWsChannel, BinanceWsChannelKind},
    BinanceSubscription
};
use crate::{
    binance::Binance,
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    normalized::ws::NormalizedWsChannels,
    traits::SpecificWsBuilder,
    CexExchange
};

#[derive(Debug, Clone, Default)]
pub struct BinanceWsBuilder {
    pub channels: Vec<BinanceWsChannel>
}

impl BinanceWsBuilder {
    async fn build_from_all_instruments_util(channels: &[BinanceWsChannelKind], streams_per_connection: Option<usize>) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Binance>()
            .await?
            .take_binance_instruments(true)
            .unwrap();
        all_symbols_vec.retain(|sym| &sym.status == "TRADING");

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.symbol)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(streams_per_connection.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION));

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
}

impl SpecificWsBuilder for BinanceWsBuilder {
    type CexExchange = Binance;
    type WsChannel = BinanceWsChannel;

    /// There is a limit of 300 connections per attempt every 5 minutes per IP.
    /// (https://binance-docs.github.io/apidocs/spot/en/#limits)
    const MAX_CONNECTIONS: usize = 300;
    /// A single connection can listen to a maximum of 1024 streams.
    /// (https://binance-docs.github.io/apidocs/spot/en/#limits)
    const MAX_STREAMS_PER_CONNECTION: usize = 1024;

    fn add_channel(mut self, channel: Self::WsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    fn build_single(self) -> Self::CexExchange {
        let mut subscription = BinanceSubscription::new();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Binance::new_ws_subscription(subscription)
    }

    fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let stream_size = if self.channels.len() <= Self::MAX_CONNECTIONS { 1 } else { self.channels.len() / Self::MAX_CONNECTIONS };
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

    fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let chunks = self
            .channels
            .chunks(connections_per_stream.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION))
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

    async fn build_from_all_instruments<'a>(
        channels: &'a [<Self::WsChannel as crate::traits::SpecificWsChannel>::ChannelKind],
        streams_per_connection: Option<usize>,
        _: Option<CexExchange>
    ) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let this = Self::build_from_all_instruments_util(channels, streams_per_connection).await?;

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

    fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, _: Option<CexExchange>) -> eyre::Result<Self>
    where
        Self: Sized
    {
        let mut this = Self { channels: Vec::new() };

        map.into_iter().try_for_each(|channel| {
            let this_channel: BinanceWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
