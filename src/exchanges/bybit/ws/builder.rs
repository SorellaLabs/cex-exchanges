use super::{
    channels::{BybitWsChannel, BybitWsChannelKind},
    BybitSubscription
};
use crate::{
    bybit::{Bybit, BybitTradingType},
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    normalized::ws::NormalizedWsChannels,
    traits::SpecificWsBuilder,
    CexExchange
};

#[derive(Debug, Clone, Default)]
pub struct BybitWsBuilder {
    pub channels: Vec<BybitWsChannel>
}

impl BybitWsBuilder {
    async fn build_from_all_instruments_util(channels: &[BybitWsChannelKind], streams_per_connection: Option<usize>) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Bybit>()
            .await?
            .take_bybit_instruments(true)
            .unwrap();
        all_symbols_vec.retain(|sym| &sym.inner.status == "Trading" && matches!(sym.trading_type, BybitTradingType::Spot));

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.inner.symbol)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(streams_per_connection.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION));

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
}

impl SpecificWsBuilder for BybitWsBuilder {
    type CexExchange = Bybit;
    type WsChannel = BybitWsChannel;

    /// There is a limit of 300 connections per attempt every 5 minutes per IP.
    /// (https://binance-docs.github.io/apidocs/spot/en/#limits)
    const MAX_CONNECTIONS: usize = 300;
    /// A single connection can listen to a maximum of 10 streams.
    const MAX_STREAMS_PER_CONNECTION: usize = 10;

    fn add_channel(mut self, channel: Self::WsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    fn build_single(self) -> Self::CexExchange {
        let mut subscription = BybitSubscription::new();

        self.channels
            .into_iter()
            .for_each(|c| subscription.add_channel(c));

        Bybit::new_ws_subscription(subscription)
    }

    fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let stream_size = if self.channels.len() <= Self::MAX_CONNECTIONS { 1 } else { self.channels.len() / Self::MAX_CONNECTIONS };

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

    fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let chunks = self
            .channels
            .chunks(connections_per_stream.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION))
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
                let mut subscription = BybitSubscription::new();
                subscription.add_channel(ch);

                Bybit::new_ws_subscription(subscription)
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
            let this_channel: BybitWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
