use super::{OkexSubscription, OkexWsChannel, OkexWsChannelKind};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    normalized::ws::NormalizedWsChannels,
    okex::Okex,
    CexExchange
};

#[derive(Debug, Clone)]
pub struct OkexWsBuilder {
    pub channels:            Vec<OkexWsChannel>,
    /// proxy exchange to get on-chain addresses
    pub exch_currency_proxy: CexExchange
}

impl OkexWsBuilder {
    /// the default proxy exchange is [CexExchange::Binance]
    pub fn new(proxy: Option<CexExchange>) -> Self {
        Self { channels: Vec::new(), exch_currency_proxy: proxy.unwrap_or(CexExchange::Binance) }
    }

    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: OkexWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// splits a [OkexWsChannel] (with values) into mutliple instance of the
    /// same [OkexWsChannel], each with fewer trading pairs by
    /// 'split_channel_size'
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: OkexWsChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            OkexWsChannel::TradesAll(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| OkexWsChannel::TradesAll(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }

            OkexWsChannel::BookTicker(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| OkexWsChannel::BookTicker(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
        }

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
    /// each with size # channels / `MAX_BINANCE_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let chunks = self
            .channels
            .chunks(self.channels.len())
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

    /// builds many ws instances of the [Okex] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let chunks = self.channels.chunks(1024).collect::<Vec<_>>();

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

    /// builds a mutlistream channel with a weighted mapping (how many channels
    /// to put per stream based on their 'exchange_ranking')
    ///
    /// [(#streams, #symbols/channel), ...]
    ///
    /// ex: [(2,3), (1,10), (1, 30), (1,55)]
    /// 2 streams with 3 symbols, 1 with 10 symbols, 1 with 20 symbols, 1
    /// with 55 symbols, 'n' streams with up to 1024 channels with the rest
    ///
    /// the default proxy exchange is [CexExchange::Binance]
    pub async fn build_all_weighted(
        weighted_map: Vec<(usize, usize)>,
        channels: &[OkexWsChannelKind],
        proxy: Option<CexExchange>
    ) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        let proxy = proxy.unwrap_or(CexExchange::Binance);
        let this = Self::build_all_weighted_util(weighted_map, channels, proxy).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = OkexSubscription::new();
                subscription.add_channel(ch);

                Okex::new_ws_subscription(subscription, proxy)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_all_weighted_util(weighted_map: Vec<(usize, usize)>, channels: &[OkexWsChannelKind], proxy: CexExchange) -> eyre::Result<Self> {
        let mut this = Self::new(Some(proxy));

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Okex>()
            .await?
            .take_okex_instruments()
            .unwrap();

        all_symbols_vec.retain(|sy| sy.instrument.state == "live");

        // reverse sort by the sort order (low to high)
        all_symbols_vec.sort_by(|a, b| a.reverse_usd_vol_24hr.cmp(&b.reverse_usd_vol_24hr));

        let mut all_symbols = all_symbols_vec.into_iter();

        let mut map = weighted_map;
        map.sort_by(|a, b| b.1.cmp(&a.1));

        while let Some(nxt) = map.pop() {
            let (mut streams, num_channels) = nxt;
            while streams > 0 {
                let mut num_channels = num_channels;

                let mut symbols_chunk = Vec::new();
                while let Some(s) = all_symbols.next() {
                    symbols_chunk.push(s.instrument.instrument.try_into()?);
                    num_channels -= 1;
                    if num_channels == 0 {
                        break
                    }
                }

                let all_channels = channels
                    .iter()
                    .map(|ch| match ch {
                        OkexWsChannelKind::TradesAll => OkexWsChannel::TradesAll(symbols_chunk.clone()),
                        OkexWsChannelKind::BookTicker => OkexWsChannel::BookTicker(symbols_chunk.clone())
                    })
                    .collect::<Vec<_>>();

                this.channels.extend(all_channels);

                streams -= 1;
            }
        }

        let rest = all_symbols
            .map(|val| val.instrument.instrument.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let rest_stream_size = std::cmp::min(1024, rest.len());
        let rest_chunks = rest.chunks(rest_stream_size);

        rest_chunks.into_iter().for_each(|chk| {
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
    pub(crate) fn make_from_normalized_map(
        map: Vec<NormalizedWsChannels>,
        split_channel_size: Option<usize>,
        exch_currency_proxy: CexExchange
    ) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new(), exch_currency_proxy };

        map.into_iter().try_for_each(|channel| {
            let this_channel: OkexWsChannel = channel.try_into()?;
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
