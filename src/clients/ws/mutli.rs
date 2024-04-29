use std::{
    pin::Pin,
    task::{Context, Poll}
};

use futures::{stream::select_all, Stream, StreamExt};
use tokio::{runtime::Handle, sync::mpsc::UnboundedReceiver};

use super::{errors::WsError, WsStream};
use crate::{exchanges::normalized::ws::CombinedWsMessage, Exchange};

pub struct MutliWsStream {
    combined_streams: Pin<Box<dyn Stream<Item = CombinedWsMessage>>>,
    stream_count:     usize
}

impl MutliWsStream {
    pub fn combine_other(self, other: Self) -> Self {
        let combined_streams = Box::pin(select_all(vec![self.combined_streams, other.combined_streams]));

        Self { combined_streams, stream_count: self.stream_count + other.stream_count }
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
                    return Poll::Ready(None)
                }
            } else {
                return Poll::Ready(val)
            }
        }

        Poll::Pending
    }
}

pub struct MutliWsStreamBuilder<T> {
    exchanges: Vec<T>
}

impl<T> MutliWsStreamBuilder<T>
where
    T: Exchange + Unpin + Send + 'static
{
    pub fn new(exchanges: Vec<T>) -> Self {
        Self { exchanges }
    }

    pub async fn build_multistream(self) -> Result<MutliWsStream, WsError> {
        let mut ws_streams = futures::stream::iter(self.exchanges)
            .map(|exch| async move {
                let mut stream = WsStream::new(exch);
                stream.connect().await?;
                Ok::<_, WsError>(stream)
            })
            .buffer_unordered(10);

        let mut all_streams = Vec::new();
        while let Some(s) = ws_streams.next().await {
            all_streams.push(Box::pin(s?))
        }

        let stream_count = all_streams.len();
        let combined_streams = Box::pin(select_all(all_streams));

        Ok(MutliWsStream { combined_streams, stream_count })
    }

    pub fn build_multistream_unconnected(self) -> MutliWsStream {
        let ws_streams = self
            .exchanges
            .into_iter()
            .map(|exch| WsStream::new(exch))
            .collect::<Vec<_>>();

        let stream_count = ws_streams.len();
        let combined_streams = Box::pin(select_all(ws_streams));

        MutliWsStream { combined_streams, stream_count }
    }

    pub fn spawn_multithreaded(self, num_threads: usize, handle: Handle) -> UnboundedReceiver<CombinedWsMessage> {
        let chunk_size = if self.exchanges.len() < num_threads + 1 { 1 } else { self.exchanges.len() / num_threads + 1 };
        let exchange_chunks = self.exchanges.chunks(chunk_size);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        exchange_chunks.into_iter().for_each(|exchanges| {
            let handle = handle.clone();
            let exchanges = exchanges.to_vec();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let this_new = Self { exchanges };
                let mut ms = this_new.build_multistream_unconnected();

                let fut = async move {
                    while let Some(val) = ms.next().await {
                        tx.send(val)?;
                    }

                    Ok::<(), eyre::Report>(())
                };

                handle.block_on(fut)?;
                Ok::<(), eyre::Report>(())
            });
        });

        rx
    }
}
