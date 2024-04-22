use super::{KucoinSubscription, KucoinWsChannel, KucoinWsChannelKind};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    kucoin::Kucoin,
    normalized::ws::NormalizedWsChannels
};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
/// (https://kucoin-docs.github.io/apidocs/spot/en/#limits)
const MAX_BINANCE_STREAMS: usize = 300;
/// A single connection can listen to a maximum of 1024 streams.
/// (https://kucoin-docs.github.io/apidocs/spot/en/#limits)
const MAX_BINANCE_WS_CONNS_PER_STREAM: usize = 1024;

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

    /// splits a [KucoinWsChannel] (with values) into mutliple instance of the
    /// same [KucoinWsChannel], each with fewer trading pairs by
    /// 'split_channel_size'
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: KucoinWsChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            KucoinWsChannel::Trade(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| KucoinWsChannel::Trade(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
            KucoinWsChannel::BookTicker(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| KucoinWsChannel::BookTicker(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
        }

        self
    }

    /// builds a single ws instance of [Kucoin], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Kucoin {
        let mut subscription = KucoinSubscription::new();

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
        let stream_size = if self.channels.len() <= MAX_BINANCE_STREAMS { 1 } else { self.channels.len() / MAX_BINANCE_STREAMS };

        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = KucoinSubscription::new();
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
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Kucoin>> {
        let chunks = self
            .channels
            .chunks(MAX_BINANCE_WS_CONNS_PER_STREAM)
            .collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = KucoinSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Kucoin::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds a mutlistream channel with a weighted mapping (how many channels
    /// to put per stream based on their 'exchange_ranking')
    ///
    /// [(#streams, #symbols/channel), ...]
    ///
    /// ex: [(2,3), (1,10), (1, 30), (1,55)]
    /// 2 streams with 3 symbols, 1 with 10 symbols, 1 with 20 symbols, 1
    /// with 55 symbols, 'n' streams with up to 1024 channels with the rest
    pub async fn build_all_weighted(
        weighted_map: Vec<(usize, usize)>,
        channels: &[KucoinWsChannelKind]
    ) -> eyre::Result<MutliWsStreamBuilder<Kucoin>> {
        let this = Self::build_all_weighted_util(weighted_map, channels).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = KucoinSubscription::new();
                subscription.add_channel(ch);

                Kucoin::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_all_weighted_util(weighted_map: Vec<(usize, usize)>, channels: &[KucoinWsChannelKind]) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Kucoin>()
            .await?
            .take_kucoin_instruments()
            .unwrap();

        all_symbols_vec.retain(|sym| sym.enable_trading);

        let mut all_symbols = all_symbols_vec.into_iter();

        let mut map = weighted_map;
        map.sort_by(|a, b| b.1.cmp(&a.1));

        while let Some(nxt) = map.pop() {
            let (mut streams, num_channels) = nxt;
            while streams > 0 {
                let mut num_channels = num_channels;

                let mut symbols_chunk = Vec::new();
                while let Some(s) = all_symbols.next() {
                    symbols_chunk.push(s.symbol.try_into()?);
                    num_channels -= 1;
                    if num_channels == 0 {
                        break
                    }
                }

                let all_channels = channels
                    .iter()
                    .map(|ch| match ch {
                        KucoinWsChannelKind::Trade => KucoinWsChannel::Trade(symbols_chunk.clone()),
                        KucoinWsChannelKind::BookTicker => KucoinWsChannel::BookTicker(symbols_chunk.clone())
                    })
                    .collect::<Vec<_>>();

                this.channels.extend(all_channels);

                streams -= 1;
            }
        }

        let rest = all_symbols
            .map(|val| val.symbol.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let rest_stream_size = std::cmp::min(MAX_BINANCE_WS_CONNS_PER_STREAM, rest.len());
        let rest_chunks = rest.chunks(rest_stream_size);

        rest_chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    KucoinWsChannelKind::Trade => KucoinWsChannel::Trade(chk.to_vec()),
                    KucoinWsChannelKind::BookTicker => KucoinWsChannel::BookTicker(chk.to_vec())
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
            let this_channel: KucoinWsChannel = channel.try_into()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn len_kind(channel: &KucoinWsChannel) -> (usize, KucoinWsChannelKind) {
        match channel {
            KucoinWsChannel::Trade(vals) => (vals.len(), KucoinWsChannelKind::Trade),
            KucoinWsChannel::BookTicker(vals) => (vals.len(), KucoinWsChannelKind::BookTicker)
        }
    }

    #[tokio::test]
    async fn test_build_many_weighted_util() {
        let map = vec![(2, 3), (1, 10), (1, 30), (1, 50)];
        let channels = vec![KucoinWsChannelKind::Trade, KucoinWsChannelKind::BookTicker];

        let calculated = KucoinWsBuilder::build_all_weighted_util(map, &channels)
            .await
            .unwrap();
        let mut calculated_channels = calculated.channels.into_iter();

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 10);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 10);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 30);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 30);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 50);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 50);
        assert!(n_kind == KucoinWsChannelKind::Trade || n_kind == KucoinWsChannelKind::BookTicker);

        let rest = calculated_channels.collect::<Vec<_>>();
        assert_eq!(rest.len(), MAX_BINANCE_STREAMS - 10);
    }
}
