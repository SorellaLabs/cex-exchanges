use std::{collections::HashMap, pin::Pin};

use futures::Stream;
use owned_chunks::OwnedChunks;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{debug, info};

use super::CombinedWsMessage;
use crate::{
    clients::ws::MutliWsStream,
    exchanges::normalized::{
        types::RawTradingPair,
        ws::channels::{NormalizedWsChannelKinds, NormalizedWsChannels}
    },
    CexExchange
};

#[derive(Debug, Default, Clone)]
pub struct NormalizedExchangeBuilder {
    pub(crate) ws_exchanges: HashMap<CexExchange, HashMap<NormalizedWsChannelKinds, NormalizedWsChannels>>,
    /// proxy exchange to get symbols for exchanges that don't have a direct api
    /// link
    exch_currency_proxy:     Option<CexExchange>
}

impl NormalizedExchangeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_channels_one_exchange(mut self, exchange: CexExchange, channels: &[NormalizedWsChannelKinds]) -> Self {
        self.ws_exchanges.entry(exchange).or_insert_with(|| {
            channels
                .iter()
                .map(|ch| (*ch, NormalizedWsChannels::new_default(*ch)))
                .collect::<HashMap<_, _>>()
        });

        self
    }

    pub fn add_channels_all_exchanges(mut self, channels: &[NormalizedWsChannelKinds]) -> Self {
        let new_map = self
            .ws_exchanges
            .into_iter()
            .map(|(exch, mut chs)| {
                channels.iter().for_each(|c| {
                    chs.entry(*c)
                        .or_insert_with(|| NormalizedWsChannels::new_default(*c));
                });

                (exch, chs)
            })
            .collect::<HashMap<_, _>>();

        self.ws_exchanges = new_map;
        self
    }

    /// sets the proxy exchange to get symbols for exchanges that don't have a
    /// direct api link
    pub fn exchange_currency_proxy(mut self, exch_currency_proxy: CexExchange) -> Self {
        self.exch_currency_proxy = Some(exch_currency_proxy);
        self
    }

    /// adds trading pairs to all channels for all exchanges
    pub fn add_pairs_all_channels_all_exchanges<S>(
        &mut self,
        exchange: &[CexExchange],
        channels: &[NormalizedWsChannelKinds],
        pairs: &[RawTradingPair]
    ) {
        exchange
            .iter()
            .for_each(|exch| self.add_pairs_all_channels(*exch, channels, pairs));
    }

    /// adds trading pairs to a channels in all exchanges
    pub fn add_pairs_single_channel_all_exchanges(&mut self, exchange: &[CexExchange], channel: NormalizedWsChannelKinds, pairs: &[RawTradingPair]) {
        exchange
            .iter()
            .for_each(|exch| self.add_pairs_single_channel(*exch, channel, pairs));
    }

    /// adds trading pairs to all channels
    pub fn add_pairs_all_channels(&mut self, exchange: CexExchange, channels: &[NormalizedWsChannelKinds], pairs: &[RawTradingPair]) {
        let entry = self.ws_exchanges.entry(exchange).or_default();

        channels.iter().for_each(|c| {
            let channel_kind: NormalizedWsChannelKinds = *c;
            entry
                .entry(channel_kind)
                .or_insert(NormalizedWsChannels::new_default(channel_kind))
                .add_pairs(exchange, pairs);
        });
    }

    /// adds trading pairs to a channel
    pub fn add_pairs_single_channel(&mut self, exchange: CexExchange, channel: NormalizedWsChannelKinds, pairs: &[RawTradingPair]) {
        let entry = self.ws_exchanges.entry(exchange).or_default();

        let channel_kind: NormalizedWsChannelKinds = channel;
        entry
            .entry(channel_kind)
            .or_insert(NormalizedWsChannels::new_default(channel_kind))
            .add_pairs(exchange, pairs);
    }

    /// builds the multistream ws client
    pub fn build_all_multistream(self, max_retries: Option<u64>, connections_per_stream: Option<usize>) -> eyre::Result<Option<MutliWsStream>> {
        let mut multistream_ws: Option<MutliWsStream> = None;

        self.ws_exchanges.into_iter().try_for_each(|(exch, map)| {
            let channel_map = map
                .into_values()
                .flat_map(|channel| channel.make_many_single())
                .collect::<Vec<_>>();

            let new_stream = exch.build_multistream_ws_from_normalized(channel_map, max_retries, connections_per_stream, self.exch_currency_proxy)?;
            if let Some(ws) = multistream_ws.take() {
                multistream_ws = Some(ws.combine_other(new_stream))
            } else {
                multistream_ws = Some(new_stream)
            }

            Ok(()) as eyre::Result<()>
        })?;

        Ok(multistream_ws)
    }

    /// builds the multithreaded multistream ws client
    pub fn build_all_multithreaded(
        self,
        number_threads: usize,
        max_retries: Option<u64>,
        connections_per_stream: Option<usize>
    ) -> eyre::Result<Option<Pin<Box<dyn Stream<Item = CombinedWsMessage> + Send>>>> {
        let all_streams = self
            .ws_exchanges
            .into_iter()
            .map(|(exch, map)| {
                let channel_map = map
                    .into_values()
                    .flat_map(|channel| channel.make_many_single())
                    .collect::<Vec<_>>();

                debug!(target: "cex-exchanges::live-stream",exchange=?exch, "made {} channels", channel_map.len());

                let streams = exch.build_multistream_unconnected_raw_ws_from_normalized(
                    channel_map,
                    self.exch_currency_proxy,
                    max_retries,
                    connections_per_stream
                )?;

                debug!(target: "cex-exchanges::live-stream",exchange=?exch, "made {} streams", streams.len());

                Ok::<_, eyre::ErrReport>(streams)

                // let new_stream =
                // exch.build_multithreaded_multistream_ws_from_normalized(
                //     channel_map,
                //     self.exch_currency_proxy,
                //     max_retries,
                //     connections_per_stream,
                //     number_threads
                // )?;
                // Ok::<_, eyre::Report>(UnboundedReceiverStream::new(new_stream))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        info!(target: "cex-exchanges::live-stream", "made {} total streams for all exchanges", all_streams.len());

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        if !all_streams.is_empty() {
            let chunk_size = (all_streams.len() as f64 / number_threads as f64).ceil() as usize;

            let stream_chks = all_streams.owned_chunks(chunk_size); //.take(chunk_size).collect::<Vec<_>>();

            debug!(target: "cex-exchanges::live-stream", "chunking streams by size {}", stream_chks.len());

            stream_chks.into_iter().for_each(|chk| {
                debug!(target: "cex-exchanges::live-stream", "made {} streams in stream chunk", chk.len());
                let stream_chk = chk.collect::<Vec<_>>();
                let tx = tx.clone();
                let multi = MutliWsStream::build_from_raw(stream_chk);
                multi.spawn_on_new_thread(tx.clone());
            });

            Ok(Some(Box::pin(UnboundedReceiverStream::new(rx))))
        } else {
            Ok(None)
        }
    }
}
