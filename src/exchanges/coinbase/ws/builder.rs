use super::{CoinbaseSubscription, CoinbaseWsChannel, CoinbaseWsChannelKind};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    coinbase::Coinbase,
    normalized::ws::NormalizedWsChannels
};

#[derive(Debug, Clone, Default)]
pub struct CoinbaseWsBuilder {
    pub channels: Vec<CoinbaseWsChannel>
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

    /// builds a single ws instance of [Coinbase], handling all channels on 1
    /// stream
    pub fn build_single(self) -> Coinbase {
        let mut sub = CoinbaseSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Coinbase::new_ws_subscription(sub)
    }

    /// builds many ws instances of the [Coinbase] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each with size # channels / `MAX_BINANCE_STREAMS` (300),
    ///
    /// WARNING: too many channels may break the stream
    pub fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        let chunks = self
            .channels
            .chunks(self.channels.len())
            .collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = CoinbaseSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Coinbase::new_ws_subscription(subscription)
            })
            .collect();

        Ok(MutliWsStreamBuilder::new(split_exchange))
    }

    /// builds many ws instances of the [Coinbase] as the inner streams of
    /// [MutliWsStreamBuilder], splitting the channels into different streams,
    /// each of size 1024
    pub fn build_many_packed(self) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        let chunks = self.channels.chunks(1024).collect::<Vec<_>>();

        let split_exchange = chunks
            .into_iter()
            .map(|chk| {
                let mut subscription = CoinbaseSubscription::new();
                chk.iter()
                    .for_each(|ch| subscription.add_channel(ch.clone()));

                Coinbase::new_ws_subscription(subscription)
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
        channels: &[CoinbaseWsChannelKind]
    ) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        let this = Self::build_all_weighted_util(weighted_map, channels).await?;

        let all_streams = this
            .channels
            .into_iter()
            .map(|ch| {
                let mut subscription = CoinbaseSubscription::new();
                subscription.add_channel(ch);

                Coinbase::new_ws_subscription(subscription)
            })
            .collect::<Vec<_>>();

        Ok(MutliWsStreamBuilder::new(all_streams))
    }

    async fn build_all_weighted_util(weighted_map: Vec<(usize, usize)>, channels: &[CoinbaseWsChannelKind]) -> eyre::Result<Self> {
        let mut this = Self::default();

        let mut all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Coinbase>()
            .await?
            .take_coinbase_instruments()
            .unwrap();

        all_symbols_vec.retain(|sy| sy.status == "online");

        // reverse sort by the sort order (low to high)
        all_symbols_vec.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

        let mut all_symbols = all_symbols_vec.into_iter();

        let mut map = weighted_map;
        map.sort_by(|a, b| b.1.cmp(&a.1));

        while let Some(nxt) = map.pop() {
            let (mut streams, num_channels) = nxt;
            while streams > 0 {
                let mut num_channels = num_channels;

                let mut symbols_chunk = Vec::new();
                while let Some(s) = all_symbols.next() {
                    symbols_chunk.push(s.id.try_into()?);
                    num_channels -= 1;
                    if num_channels == 0 {
                        break
                    }
                }

                let all_channels = channels
                    .iter()
                    .map(|ch| match ch {
                        CoinbaseWsChannelKind::Matches => CoinbaseWsChannel::Matches(symbols_chunk.clone()),
                        CoinbaseWsChannelKind::Ticker => CoinbaseWsChannel::Ticker(symbols_chunk.clone()),
                        CoinbaseWsChannelKind::Status => CoinbaseWsChannel::Status
                    })
                    .collect::<Vec<_>>();

                this.channels.extend(all_channels);

                streams -= 1;
            }
        }

        let rest = all_symbols
            .map(|val| val.id.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        let rest_stream_size = std::cmp::min(1024, rest.len());
        let rest_chunks = rest.chunks(rest_stream_size);

        rest_chunks.into_iter().for_each(|chk| {
            let all_channels = channels
                .iter()
                .map(|ch| match ch {
                    CoinbaseWsChannelKind::Matches => CoinbaseWsChannel::Matches(chk.to_vec()),
                    CoinbaseWsChannelKind::Ticker => CoinbaseWsChannel::Ticker(chk.to_vec()),
                    CoinbaseWsChannelKind::Status => CoinbaseWsChannel::Status
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
            &[RawTradingPair::new_raw("pepe_USD", '_'), RawTradingPair::new_base_quote("ETH", "usd", None)]
        );

        builder.add_pairs_single_channel(CexExchange::Coinbase, NormalizedWsChannelKinds::Trades, &[RawTradingPair::new_no_delim("wbtc-usd")]);

        let map = builder
            .ws_exchanges
            .get(&CexExchange::Coinbase)
            .unwrap()
            .into_iter()
            .map(|(_, vals)| vals.to_owned())
            .collect::<Vec<_>>();

        let calculated_builder = CoinbaseWsBuilder::make_from_normalized_map(map, Some(1)).unwrap();

        let expected_trade_channel = CoinbaseWsChannel::Matches(vec![
            CoinbaseTradingPair("WBTC-USD".to_string()),
            CoinbaseTradingPair("ETH-USD".to_string()),
            CoinbaseTradingPair("PEPE-USD".to_string()),
        ]);

        let expected_quote_channel =
            CoinbaseWsChannel::Ticker(vec![CoinbaseTradingPair("ETH-USD".to_string()), CoinbaseTradingPair("PEPE-USD".to_string())]);

        let expected_builder = CoinbaseWsBuilder { channels: vec![expected_quote_channel, expected_trade_channel] };

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
    }
}
