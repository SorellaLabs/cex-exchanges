use super::{BinanceWsChannel, BinanceWsChannelKind};
use crate::{
    binance::Binance,
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    exchanges::binance::WSS_URL,
    normalized::{types::Blockchain, ws::NormalizedWsChannels},
    CexExchange
};

#[derive(Debug, Clone, Default)]
pub struct BinanceWsBuilder {
    pub channels:            Vec<BinanceWsChannel>,
    pub channels_per_stream: Option<usize>
}

impl BinanceWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: BinanceWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// splits a [BinanceWsChannel] (with values) into mutliple instance of the
    /// same [BinanceWsChannel], each with fewer trading pairs by
    /// 'split_channel_size'
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: BinanceWsChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            BinanceWsChannel::Trade(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| BinanceWsChannel::Trade(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
            BinanceWsChannel::BookTicker(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| BinanceWsChannel::BookTicker(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
        }

        self
    }

    /// sets the number of channels
    pub fn set_channels_per_stream(mut self, channels_per_stream: usize) -> Self {
        self.channels_per_stream = Some(channels_per_stream);
        self
    }

    /// builds a single ws instance of [Binance], handling all channels on 1
    /// stream
    pub fn build(self) -> Binance {
        let base_url = WSS_URL.to_string();
        let channel_urls = self
            .channels
            .into_iter()
            .map(|c| c.build_url())
            .collect::<Vec<_>>();

        let url = format!("{base_url}{}", channel_urls.join("/"));

        Binance::new_ws_subscription(url)
    }

    /// builds many ws instances of the [Binance] as the inner streams of
    /// [MutliWsStreamBuilder] IFF 'channels_per_stream' is set, splitting
    /// channels by the specified number
    pub fn build_many(self) -> eyre::Result<MutliWsStreamBuilder<Binance>> {
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let channel_urls = chk.iter().map(|c| c.build_url()).collect::<Vec<_>>();

                    let url = format!("{WSS_URL}{}", channel_urls.join("/"));

                    Binance::new_ws_subscription(url)
                })
                .collect();

            Ok(MutliWsStreamBuilder::new(split_exchange))
        } else {
            Err(eyre::ErrReport::msg("'channels_per_stream' was not set".to_string()))
        }
    }

    /// builds a mutlistream channel with a weighted mapping (how many channels
    /// to put per stream based on their 'exchange_ranking')
    ///
    /// [(#streams, #symbols/channel), ...]
    ///
    /// ex: [(2,3), (1,10), (1, 30), (1,55)]
    /// 2 streams with 3 symbols, 1 with 10 symbols, 1 with 20 symbols, 1
    /// with 55 symbols, 1 with the rest
    pub async fn build_ranked_weighted_all_symbols(
        weighted_map: Vec<(usize, usize)>,
        channels: &[BinanceWsChannelKind],
        blockchain: Option<Blockchain>
    ) -> eyre::Result<MutliWsStreamBuilder<Binance>> {
        let this = Self::build_ranked_weighted_all_symbols_util(weighted_map, channels, blockchain).await?;
        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let channel_url = ch.build_url();

                let url = format!("{WSS_URL}{channel_url}");

                Binance::new_ws_subscription(url)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_ranked_weighted_all_symbols_util(
        weighted_map: Vec<(usize, usize)>,
        channels: &[BinanceWsChannelKind],
        blockchain: Option<Blockchain>
    ) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Binance>()
            .await?
            .take_binance_instruments()
            .unwrap();

        if let Some(bk) = blockchain {
            all_symbols_vec.retain(|instr| {
                instr
                    .clone()
                    .normalize()
                    .blockchains
                    .iter()
                    .any(|(chain, _)| *chain == bk)
            });
        }

        all_symbols_vec.sort_by(|a, b| a.quote)

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
                        BinanceWsChannelKind::Trade => BinanceWsChannel::Trade(symbols_chunk.clone()),
                        BinanceWsChannelKind::BookTicker => BinanceWsChannel::BookTicker(symbols_chunk.clone())
                    })
                    .collect::<Vec<_>>();

                this.channels.extend(all_channels);

                streams -= 1
            }
        }

        let rest = all_symbols
            .map(|v| Ok(v.symbol.try_into()?))
            .collect::<eyre::Result<Vec<_>>>()?;
        if !rest.is_empty() {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    BinanceWsChannelKind::Trade => BinanceWsChannel::Trade(rest.clone()),
                    BinanceWsChannelKind::BookTicker => BinanceWsChannel::BookTicker(rest.clone())
                })
                .collect::<Vec<_>>();

            this.channels.extend(all_channels);
        }

        Ok(this)
    }

    /// makes the builder from the normalized builder's map
    pub(crate) fn make_from_normalized_map(
        map: Vec<NormalizedWsChannels>,
        channels_per_stream: Option<usize>,
        split_channel_size: Option<usize>
    ) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new(), channels_per_stream };

        map.into_iter().try_for_each(|channel| {
            let this_channel: BinanceWsChannel = channel.try_into()?;
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

    fn len_kind(channel: &BinanceWsChannel) -> (usize, BinanceWsChannelKind) {
        match channel {
            BinanceWsChannel::Trade(vals) => (vals.len(), BinanceWsChannelKind::Trade),
            BinanceWsChannel::BookTicker(vals) => (vals.len(), BinanceWsChannelKind::BookTicker)
        }
    }

    #[tokio::test]
    async fn test_build_ranked_weighted_all_symbols_util() {
        let map = vec![(2, 3), (1, 10), (1, 30), (1, 50)];
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let calculated = BinanceWsBuilder::build_ranked_weighted_all_symbols_util(map, &channels, None)
            .await
            .unwrap();
        let mut calculated_channels = calculated.channels.into_iter();

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 3);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 10);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 10);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 30);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 30);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 50);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        let (n_len, n_kind) = len_kind(&calculated_channels.next().unwrap());
        assert_eq!(n_len, 50);
        assert!(n_kind == BinanceWsChannelKind::Trade || n_kind == BinanceWsChannelKind::BookTicker);

        assert!(calculated_channels.next().is_some());
        assert!(calculated_channels.next().is_some());

        assert!(calculated_channels.next().is_none());
    }
}
