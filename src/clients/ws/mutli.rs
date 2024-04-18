use std::{
    pin::Pin,
    task::{Context, Poll}
};

use futures::{future::join_all, stream::select_all, Stream, StreamExt};

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
        let ws_streams = join_all(self.exchanges.into_iter().map(|exch| async move {
            let mut stream = WsStream::new(exch);
            stream.connect().await?;
            Ok(stream)
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, WsError>>()?;

        let stream_count = ws_streams.len();
        let combined_streams = Box::pin(select_all(ws_streams));

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
}
