use super::{
    channels::{OkexWsChannel, OkexWsChannelKind},
    OkexSubscription
};
use crate::{
    clients::ws::MutliWsStreamBuilder,
    normalized::{types::InstrumentFilter, ws::NormalizedWsChannels},
    okex::Okex,
    traits::{SpecificWsBuilder, SpecificWsSubscription},
    CexExchange
};

#[derive(Debug, Clone)]
pub struct OkexWsBuilder {
    pub channels:            Vec<OkexWsChannel>,
    /// proxy exchange to get on-chain addresses
    pub exch_currency_proxy: CexExchange
}

impl OkexWsBuilder {
    /// the default proxy exchange is [CexExchange::Okex]
    pub fn new(proxy: Option<CexExchange>) -> Self {
        Self { channels: Vec::new(), exch_currency_proxy: proxy.unwrap_or(CexExchange::Okex) }
    }

    async fn build_from_all_instruments_util(
        channels: &[OkexWsChannelKind],
        streams_per_connection: Option<usize>,
        proxy: Option<CexExchange>
    ) -> eyre::Result<Self> {
        let mut this = Self::new(proxy);

        let all_symbols = this
            .exch_currency_proxy
            .get_all_instruments(Some(InstrumentFilter::Active))
            .await?;

        let rest = all_symbols
            .into_iter()
            .map(|val| val.trading_pair.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let chunks = rest.chunks(streams_per_connection.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION));

        chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    OkexWsChannelKind::TradesAll => OkexWsChannel::TradesAll(chk.to_vec()),
                    OkexWsChannelKind::BookTicker => OkexWsChannel::BookTicker(chk.to_vec())
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        });

        Ok(this)
    }
}

impl SpecificWsBuilder for OkexWsBuilder {
    type CexExchange = Okex;
    type WsChannel = OkexWsChannel;

    /// There is a limit of 300 connections per attempt every 5 minutes per IP.
    const MAX_CONNECTIONS: usize = 300;
    /// A single connection can listen to a maximum of 100 streams.
    const MAX_STREAMS_PER_CONNECTION: usize = 100;

    fn add_channel(mut self, channel: Self::WsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    fn build_single(self) -> Self::CexExchange {
        let mut sub = OkexSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Okex::new_ws_subscription(sub, self.exch_currency_proxy)
    }

    fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let stream_size = if self.channels.len() <= Self::MAX_CONNECTIONS { 1 } else { self.channels.len() / Self::MAX_CONNECTIONS };

        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = OkexSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Okex::new_ws_subscription(subscription, self.exch_currency_proxy)
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
                let mut subscription = OkexSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Okex::new_ws_subscription(subscription, self.exch_currency_proxy)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    async fn build_from_all_instruments<'a>(
        channels: &'a [<Self::WsChannel as crate::traits::SpecificWsChannel>::ChannelKind],
        streams_per_connection: Option<usize>,
        exch_currency_proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let this = Self::build_from_all_instruments_util(channels, streams_per_connection, exch_currency_proxy).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = OkexSubscription::new();
                subscription.add_channel(ch);

                Okex::new_ws_subscription(subscription, this.exch_currency_proxy)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, exch_currency_proxy: Option<CexExchange>) -> eyre::Result<Self>
    where
        Self: Sized
    {
        let mut this = Self { channels: Vec::new(), exch_currency_proxy: exch_currency_proxy.unwrap_or(CexExchange::Okex) };

        map.into_iter().try_for_each(|channel| {
            let this_channel: OkexWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
