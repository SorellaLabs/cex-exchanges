use super::BinanceWsChannel;
use crate::{binance::Binance, clients::ws::MutliWsStreamBuilder, exchanges::binance::WSS_URL, normalized::ws::NormalizedWsChannels};

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
        let base_url = WSS_URL.to_string();
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let channel_urls = chk.iter().map(|c| c.build_url()).collect::<Vec<_>>();

                    let url = format!("{base_url}{}", channel_urls.join("/"));

                    Binance::new_ws_subscription(url)
                })
                .collect();

            Ok(MutliWsStreamBuilder::new(split_exchange))
        } else {
            Err(eyre::ErrReport::msg("'channels_per_stream' was not set".to_string()))
        }
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
