use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{errors::WsError, WsStream, WsStreamConfig};
use crate::{exchanges::normalized::ws::CombinedWsMessage, Exchange};

pub struct MutliWsStream {
    combined_streams: Pin<Box<dyn Stream<Item = CombinedWsMessage> + Send>>,
    stream_count: usize,
}

impl MutliWsStream {
    pub fn combine_other(self, other: Self) -> Self {
        let combined_streams = Box::pin(futures::stream::select_all(vec![self.combined_streams, other.combined_streams]));

        Self { combined_streams, stream_count: self.stream_count + other.stream_count }
    }

    pub fn stream_count(&self) -> usize {
        self.stream_count
    }

    pub(crate) fn spawn_on_new_thread(self, tx: UnboundedSender<CombinedWsMessage>) {
        std::thread::spawn(move || {
            let thread_rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;

            thread_rt.block_on(self.run_with_sender(tx))?;
            Ok::<(), eyre::Report>(())
        });
    }

    pub async fn run_with_sender(mut self, tx: UnboundedSender<CombinedWsMessage>) -> eyre::Result<()> {
        while let Some(val) = self.next().await {
            tx.send(val)?;
        }

        Ok(())
    }

    pub(crate) fn build_from_raw(raw_streams: Vec<Pin<Box<dyn Stream<Item = CombinedWsMessage> + Send>>>) -> Self {
        let stream_count = raw_streams.len();
        let combined_streams = Box::pin(futures::stream::select_all(raw_streams));
        Self { stream_count, combined_streams }
    }
}

impl Stream for MutliWsStream {
    type Item = CombinedWsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        while let Poll::Ready(val) = this.combined_streams.poll_next_unpin(cx) {
            if val.is_none() {
                this.stream_count -= 1;
                if this.stream_count == 0 {
                    return Poll::Ready(None);
                }
            } else {
                return Poll::Ready(val);
            }
        }

        Poll::Pending
    }
}

pub struct MultiWsStreamBuilder<T> {
    exchanges: Vec<T>,
}

impl<T> MultiWsStreamBuilder<T>
where
    T: Exchange + Unpin + Debug + Send + 'static,
{
    pub fn new(exchanges: Vec<T>) -> Self {
        Self { exchanges }
    }

    pub async fn build_multistream(self, config: WsStreamConfig) -> Result<MutliWsStream, WsError> {
        let ws_streams = futures::stream::iter(self.exchanges)
            .map(|exch| async move {
                let mut stream = WsStream::new(exch, config);
                stream.connect().await?;
                Ok::<_, WsError>(stream)
            })
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let stream_count = ws_streams.len();
        let combined_streams = Box::pin(futures::stream::select_all(ws_streams));

        Ok(MutliWsStream { combined_streams, stream_count })
    }

    pub fn build_multistream_unconnected(self, config: WsStreamConfig) -> MutliWsStream {
        let ws_streams = self
            .exchanges
            .into_iter()
            .map(|exch| WsStream::new(exch, config))
            .collect::<Vec<_>>();

        let stream_count = ws_streams.len();
        let combined_streams = Box::pin(futures::stream::select_all(ws_streams));

        MutliWsStream { combined_streams, stream_count }
    }

    pub(crate) fn build_multistream_unconnected_raw(self, config: WsStreamConfig) -> Vec<Pin<Box<dyn Stream<Item = CombinedWsMessage> + Send>>> {
        self.exchanges
            .into_iter()
            .map(|exch| Box::pin(WsStream::new(exch, config)) as Pin<Box<dyn Stream<Item = CombinedWsMessage> + Send>>)
            .collect::<Vec<_>>()
    }

    pub fn spawn_multithreaded(self, num_threads: usize, config: WsStreamConfig) -> UnboundedReceiver<CombinedWsMessage> {
        let chunk_size = if self.exchanges.len() < num_threads + 1 { 1 } else { self.exchanges.len() / num_threads + 1 };
        let exchange_chunks = self.exchanges.chunks(chunk_size);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        exchange_chunks.into_iter().for_each(|exchanges| {
            let exchanges = exchanges.to_vec();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let thread_rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                let this_new = Self { exchanges };
                let ms = this_new.build_multistream_unconnected(config);

                thread_rt.block_on(ms.run_with_sender(tx))?;
                Ok::<(), eyre::Report>(())
            });
        });

        rx
    }
}
