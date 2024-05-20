use super::{CoinbaseSubscription, CoinbaseWsChannel, CoinbaseWsChannelKind};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    coinbase::Coinbase,
    normalized::ws::NormalizedWsChannels
};

/// There is a limit of 300 connections per attempt every 5 minutes per IP.
const MAX_COINBASE_STREAMS: usize = 8;
/// A single connection can listen to a maximum of 100 streams.
const MAX_COINBASE_WS_CONNS_PER_STREAM: usize = 100;

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
        let stream_size = if self.channels.len() <= MAX_COINBASE_STREAMS { 1 } else { self.channels.len() / MAX_COINBASE_STREAMS };

        let chunks = self.channels.chunks(stream_size).collect::<Vec<_>>();

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
    pub fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        let chunks = self
            .channels
            .chunks(connections_per_stream.unwrap_or(MAX_COINBASE_WS_CONNS_PER_STREAM))
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

    /// builds a mutlistream channel from all active instruments
    pub async fn build_from_all_instruments(
        channels: &[CoinbaseWsChannelKind],
        connections_per_stream: Option<usize>
    ) -> eyre::Result<MutliWsStreamBuilder<Coinbase>> {
        let this = Self::build_from_all_instruments_util(channels, connections_per_stream).await?;

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

    async fn build_from_all_instruments_util(channels: &[CoinbaseWsChannelKind], connections_per_stream: Option<usize>) -> eyre::Result<Self> {
        let mut this = Self::default();

        let all_symbols_vec = ExchangeApi::new()
            .all_instruments::<Coinbase>()
            .await?
            .take_coinbase_instruments(true)
            .unwrap();

        let all_symbols = all_symbols_vec
            .into_iter()
            .map(|val| val.id)
            .collect::<Vec<_>>();

        let chunks = all_symbols.chunks(connections_per_stream.unwrap_or(MAX_COINBASE_WS_CONNS_PER_STREAM));

        chunks.into_iter().for_each(|chk| {
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
    pub(crate) fn make_from_normalized_map(map: Vec<NormalizedWsChannels>) -> eyre::Result<Self> {
        let mut this = Self { channels: Vec::new() };

        map.into_iter().try_for_each(|channel| {
            let this_channel: CoinbaseWsChannel = channel.try_into()?;
            this = this.clone().add_channel(this_channel);
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
            .iter()
            .map(|(_, vals)| vals.to_owned())
            .collect::<Vec<_>>();

        let calculated_builder = CoinbaseWsBuilder::make_from_normalized_map(map).unwrap();

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
