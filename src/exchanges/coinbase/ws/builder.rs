use super::{
    channels::{CoinbaseWsChannel, CoinbaseWsChannelKind},
    CoinbaseSubscription
};
use crate::{
    clients::{rest_api::ExchangeApi, ws::MutliWsStreamBuilder},
    coinbase::Coinbase,
    normalized::ws::NormalizedWsChannels,
    traits::{SpecificWsBuilder, SpecificWsSubscription},
    CexExchange
};

#[derive(Debug, Clone, Default)]
pub struct CoinbaseWsBuilder {
    pub channels: Vec<CoinbaseWsChannel>
}

impl CoinbaseWsBuilder {
    async fn build_from_all_instruments_util(channels: &[CoinbaseWsChannelKind], streams_per_connection: Option<usize>) -> eyre::Result<Self> {
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

        let chunks = all_symbols.chunks(streams_per_connection.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION));

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
}

impl SpecificWsBuilder for CoinbaseWsBuilder {
    type CexExchange = Coinbase;
    type WsChannel = CoinbaseWsChannel;

    /// There is a limit of 8 connections per attempt every 5 minutes per IP.
    const MAX_CONNECTIONS: usize = 8;
    /// A single connection can listen to a maximum of 100 streams.
    const MAX_STREAMS_PER_CONNECTION: usize = 100;

    fn add_channel(mut self, channel: Self::WsChannel) -> Self {
        self.channels.push(channel);
        self
    }

    fn build_single(self) -> Self::CexExchange {
        let mut sub = CoinbaseSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Coinbase::new_ws_subscription(sub)
    }

    fn build_many_distributed(self) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let stream_size = if self.channels.len() <= Self::MAX_CONNECTIONS { 1 } else { self.channels.len() / Self::MAX_CONNECTIONS };

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

    fn build_many_packed(self, connections_per_stream: Option<usize>) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let chunks = self
            .channels
            .chunks(connections_per_stream.unwrap_or(Self::MAX_STREAMS_PER_CONNECTION))
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

    async fn build_from_all_instruments<'a>(
        channels: &'a [<Self::WsChannel as crate::traits::SpecificWsChannel>::ChannelKind],
        streams_per_connection: Option<usize>,
        _: Option<CexExchange>
    ) -> eyre::Result<MutliWsStreamBuilder<Self::CexExchange>> {
        let this = Self::build_from_all_instruments_util(channels, streams_per_connection).await?;

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

    fn make_from_normalized_map(map: Vec<NormalizedWsChannels>, _: Option<CexExchange>) -> eyre::Result<Self>
    where
        Self: Sized
    {
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

    use super::*;
    use crate::{
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

        let calculated_builder = CoinbaseWsBuilder::make_from_normalized_map(map, None).unwrap();

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
