use super::{CoinbaseSubscription, CoinbaseWsChannel};
use crate::{clients::ws::MutliWsStreamBuilder, coinbase::Coinbase, normalized::ws::NormalizedWsChannels};

#[derive(Debug, Clone, Default)]
pub struct CoinbaseWsBuilder {
    pub channels:            Vec<CoinbaseWsChannel>,
    pub channels_per_stream: Option<usize>
}

impl CoinbaseWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: CoinbaseWsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// splits a [CoinbaseWsChannel] (with values) into mutliple instance of the
    /// same [CoinbaseWsChannel], each with fewer trading pairs by
    /// 'split_channel_size'
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: CoinbaseWsChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            CoinbaseWsChannel::Status => self.channels.push(channel),
            CoinbaseWsChannel::Matches(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| CoinbaseWsChannel::Matches(chk.to_vec()))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }

            CoinbaseWsChannel::Ticker(vals) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(1), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| CoinbaseWsChannel::Ticker(chk.to_vec()))
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

    /// builds a single ws instance of [Coinbase], handling all channels on 1
    /// stream
    pub fn build(self) -> Coinbase {
        let mut sub = CoinbaseSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Coinbase::new_ws_subscription(sub)
    }

    /// builds many ws instances of the [Coinbase] as the inner streams of
    /// [MutliWsStreamBuilder] IFF 'channels_per_stream' is set, splitting
    /// channels by the specified number
    pub fn build_many(self) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let mut sub = CoinbaseSubscription::new();

                    chk.iter().for_each(|c| sub.add_channel(c.clone()));

                    Coinbase::new_ws_subscription(sub)
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
            let this_channel: CoinbaseWsChannel = channel.try_into()?;
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

    use crate::{
        coinbase::ws::{CoinbaseWsBuilder, CoinbaseWsChannel},
        exchanges::{
            coinbase::CoinbaseTradingPair,
            normalized::{types::RawTradingPair, ws::NormalizedWsChannelKinds}
        },
        normalized::ws::NormalizedExchangeBuilder,
        CexExchange
    };

    #[test]
    fn test_make_from_normalized_builder() {
        let mut builder = NormalizedExchangeBuilder::default()
            .add_channels_one_exchange(CexExchange::Coinbase, &[NormalizedWsChannelKinds::Quotes, NormalizedWsChannelKinds::Trades]);

        builder.add_pairs_all_channels(
            CexExchange::Coinbase,
            &[NormalizedWsChannelKinds::Quotes, NormalizedWsChannelKinds::Trades],
            &[RawTradingPair::new_raw("pepe_USD", '_'), RawTradingPair::new_base_quote("ETH", "usd")]
        );

        builder.add_pairs_single_channel(CexExchange::Coinbase, NormalizedWsChannelKinds::Trades, &[RawTradingPair::new_no_delim("wbtc-usd")]);

        let map = builder
            .ws_exchanges
            .get(&CexExchange::Coinbase)
            .unwrap()
            .into_iter()
            .map(|(_, vals)| vals.to_owned())
            .collect::<Vec<_>>();

        let calculated_builder = CoinbaseWsBuilder::make_from_normalized_map(map, Some(1), None).unwrap();

        let expected_trade_channel = CoinbaseWsChannel::Matches(vec![
            CoinbaseTradingPair("WBTC-USD".to_string()),
            CoinbaseTradingPair("ETH-USD".to_string()),
            CoinbaseTradingPair("PEPE-USD".to_string()),
        ]);

        let expected_quote_channel =
            CoinbaseWsChannel::Ticker(vec![CoinbaseTradingPair("ETH-USD".to_string()), CoinbaseTradingPair("PEPE-USD".to_string())]);

        let expected_builder =
            CoinbaseWsBuilder { channels: vec![expected_quote_channel, expected_trade_channel], channels_per_stream: Some(1) };

        let assert1 = calculated_builder
            .channels
            .clone()
            .iter_mut()
            .any(|channel| {
                if let (CoinbaseWsChannel::Ticker(mut vals1), CoinbaseWsChannel::Ticker(vals2)) = (expected_builder.channels[0].clone(), channel) {
                    vals1.sort_by(|a, b| a.0.cmp(&b.0));
                    vals2.sort_by(|a, b| a.0.cmp(&b.0));
                    &vals1 == vals2
                } else {
                    false
                }
            });
        assert!(assert1);

        let assert2 = calculated_builder
            .channels
            .clone()
            .iter_mut()
            .any(|channel| {
                if let (CoinbaseWsChannel::Matches(mut vals1), CoinbaseWsChannel::Matches(vals2)) = (expected_builder.channels[1].clone(), channel) {
                    vals1.sort_by(|a, b| a.0.cmp(&b.0));
                    vals2.sort_by(|a, b| a.0.cmp(&b.0));
                    &vals1 == vals2
                } else {
                    false
                }
            });
        assert!(assert2);

        assert_eq!(expected_builder.channels_per_stream, calculated_builder.channels_per_stream);
    }
}
