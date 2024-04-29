use super::{OkexSubscription, OkexWsChannel, OkexWsChannelKind};
use crate::{clients::ws::MutliWsStreamBuilder, normalized::ws::NormalizedWsChannels, okex::Okex, CexExchange};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
const MAX_OKEX_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 1024 streams.
const MAX_OKEX_WS_CONNS_PER_STREAM: usize = 1024;

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

    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: OkexWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// builds a single ws instance of [Okex], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Okex {
        let mut sub = OkexSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Okex::new_ws_subscription(sub, self.exch_currency_proxy)
    }

    /// builds many ws instances of the [Okex] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each with size # channels / `MAX_OKEX_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let stream_size = if self.channels.len() <= MAX_OKEX_STREAMS { 1 } else { self.channels.len() / MAX_OKEX_STREAMS };

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

    /// builds many ws instances of the [Okex] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let chunks = self
            .channels
            .chunks(MAX_OKEX_WS_CONNS_PER_STREAM)
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

    /// builds a mutlistream channel from all active instruments
    pub async fn build_from_all_instruments(channels: &[OkexWsChannelKind], proxy: Option<CexExchange>) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let this = Self::build_from_all_instruments_util(channels, proxy).await?;

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

    async fn build_from_all_instruments_util(channels: &[OkexWsChannelKind], proxy: Option<CexExchange>) -> eyre::Result<Self> {
        let mut this = Self::new(proxy);

        let mut all_symbols = this.exch_currency_proxy.get_all_instruments().await?;
        all_symbols.retain(|sym| sym.active);

        let rest = all_symbols
            .into_iter()
            .map(|val| val.trading_pair.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let chunks = rest.chunks(MAX_OKEX_WS_CONNS_PER_STREAM);

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

    /// makes the builder from the normalized builder's map
    pub(crate) fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, exch_currency_proxy: CexExchange) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new(), exch_currency_proxy };

        map.into_iter().try_for_each(|channel| {
            let this_channel: OkexWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
            Ok(()) as eyre::Result<()>
        })?;

        Ok(this)
    }
}
