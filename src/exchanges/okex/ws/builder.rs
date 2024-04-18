use super::{OkexSubscription, OkexWsChannel};
use crate::{clients::ws::MutliWsStreamBuilder, normalized::ws::NormalizedWsChannels, okex::Okex, CexExchange};

#[derive(Debug, Clone)]
pub struct OkexWsBuilder {
    pub channels:            Vec<OkexWsChannel>,
    /// sets the number of channels per stream
    pub channels_per_stream: Option<usize>,
    pub exch_currency_proxy: CexExchange
}

impl OkexWsBuilder {
    pub fn new(exch_currency_proxy: CexExchange) -> Self {
        Self { channels: Vec::new(), channels_per_stream: None, exch_currency_proxy }
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

    /// sets the number of channels per stream
    pub fn set_channels_per_stream(mut self, channels_per_stream: usize) -> Self {
        self.channels_per_stream = Some(channels_per_stream);
        self
    }

    /// builds a single ws instance of [Okex], handling all channels on 1
    /// stream
    pub fn build(self) -> Okex {
        let mut sub = OkexSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Okex::new_ws_subscription(sub, self.exch_currency_proxy)
    }

    /// builds many ws instances of the [Okex] as the inner streams of
    /// [MutliWsStreamBuilder] IFF 'channels_per_stream' is set, splitting
    /// channels by the specified number
    pub fn build_many(self) -> eyre::Result<MutliWsStreamBuilder<Okex>> {
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let mut sub = OkexSubscription::new();

                    chk.iter().for_each(|c| sub.add_channel(c.clone()));

                    Okex::new_ws_subscription(sub, self.exch_currency_proxy)
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
        split_channel_size: Option<usize>,
        exch_currency_proxy: CexExchange
    ) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new(), channels_per_stream, exch_currency_proxy };

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
