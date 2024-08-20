use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll}
};

use futures::{Future, FutureExt, SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};

use super::WsError;
use crate::{
    clients::ws::critical::CriticalWsMessage,
    exchanges::normalized::ws::{CombinedWsMessage, MessageOrPing},
    Exchange
};
type ReconnectFuture = Option<Pin<Box<dyn Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>> + Send>>>;

type StreamConn = Pin<Box<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

pub struct WsStream<T> {
    exchange:      T,
    stream:        Option<StreamConn>,
    reconnect_fut: ReconnectFuture,
    max_retries:   Option<u64>,
    retry_count:   u64
}

impl<T> WsStream<T>
where
    T: Exchange + Send
{
    pub fn new(exchange: T, max_retries: Option<u64>) -> Self {
        Self { exchange, stream: None, reconnect_fut: None, max_retries, retry_count: 0 }
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        let ws = self.exchange.make_ws_connection().await;
        if let Err(e) = ws {
            error!(target: "cex-exchanges::live-stream", "error connecting to the {} websocket stream: {:?}", T::EXCHANGE, e);
            return Err(e)
        }

        self.stream = Some(Box::pin(ws?));
        Ok(())
    }

    fn handle_incoming(message: Message) -> Result<MessageOrPing<T>, (WsError, String)> {
        match message {
            Message::Text(msg) => {
                trace!(target: "cex-exchanges::live-stream", "recieved new message for the {} stream: {}",T::EXCHANGE, msg);

                let mut des_msg = serde_json::from_str::<T::WsMessage>(&msg).map_err(|e| (e.into(), msg.clone()))?;
                des_msg.make_critical(msg);
                Ok(MessageOrPing::new_message(des_msg))
            }
            Message::Ping(ping) => Ok(MessageOrPing::new_ping(ping)),
            Message::Binary(_) => panic!("Exchange: {} - Message::Binary", T::EXCHANGE),
            Message::Pong(_) => panic!("Exchange: {} - Message::Pong", T::EXCHANGE),
            Message::Close(_) => Ok(MessageOrPing::new_close()),
            Message::Frame(_) => panic!("Exchange: {} - Message::Frame", T::EXCHANGE)
        }
    }

    fn flush_sink_queue(stream: &mut StreamConn, cx: &mut Context<'_>) -> Result<(), WsError> {
        let mut ret = Ok(());
        loop {
            match stream.poll_flush_unpin(cx) {
                Poll::Ready(Ok(_)) => break,
                Poll::Ready(Err(e)) => {
                    ret = Err(WsError::StreamTxError(e));
                    break;
                }
                Poll::Pending => ()
            }
        }

        debug!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "finished flushing queue sink with Pong");

        ret
    }

    fn handle_retry(&mut self, msg: CombinedWsMessage) -> Poll<Option<CombinedWsMessage>> {
        let stay_same = match self.handle_bad_pair(&msg) {
            Some(true) => return Poll::Ready(None),
            Some(false) => false,
            None => true
        };
        if let Some(retries) = self.max_retries {
            if !stay_same {
                self.retry_count += 1;
            }

            if self.retry_count > retries {
                error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "retries exceeded -- EXITING");
                return Poll::Ready(None)
            }
        }

        Poll::Ready(Some(msg))
    }

    /// Some(true) => subscription is empty
    /// Some(false) => subscription is not empty
    /// None => no bad pair found
    fn handle_bad_pair(&mut self, msg: &CombinedWsMessage) -> Option<bool> {
        msg.bad_pair().map(|p| self.exchange.remove_bad_pair(p))
    }
}

impl<T> Stream for WsStream<T>
where
    T: Exchange + Debug + Send + Unpin + 'static,
    Self: Send
{
    type Item = CombinedWsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(stream) = this.stream.as_mut() {
            if let Poll::Ready(val) = stream.poll_next_unpin(cx) {
                match val {
                    Some(Ok(msg)) => match Self::handle_incoming(msg) {
                        Ok(MessageOrPing::Message(d)) => return this.handle_retry(d.into()),
                        Ok(MessageOrPing::Ping(v)) => {
                            debug!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "recieved ping");
                            if let Err(e) = stream.start_send_unpin(Message::Pong(v)) {
                                error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error sending pong");
                                this.stream = None;
                                return this.handle_retry(WsError::StreamTxError(e).normalized_with_exchange(T::EXCHANGE, None))
                            } else if let Err(e) = Self::flush_sink_queue(stream, cx) {
                                warn!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error flushing queue sink");
                                this.stream = None;
                                return this.handle_retry(e.normalized_with_exchange(T::EXCHANGE, None))
                            }

                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        Ok(MessageOrPing::Close) => {
                            this.stream = None;

                            return Poll::Pending;
                        }
                        Err((e, raw_msg)) => {
                            this.stream = None;

                            return this.handle_retry(e.normalized_with_exchange(T::EXCHANGE, Some(raw_msg)));
                        }
                    },
                    Some(Err(e)) => {
                        this.stream = None;
                        return this.handle_retry(WsError::StreamRxError(e).normalized_with_exchange(T::EXCHANGE, None))
                    }
                    None => {
                        this.stream = None;
                        return this.handle_retry(WsError::StreamTerminated.normalized_with_exchange(T::EXCHANGE, None))
                    }
                };
            }
        } else if let Some(reconnect) = this.reconnect_fut.as_mut() {
            match reconnect.poll_unpin(cx) {
                Poll::Ready(Ok(new_stream)) => {
                    info!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "successfully reconnected to stream");
                    this.stream = Some(Box::pin(new_stream));
                    this.reconnect_fut = None;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error reconnecting to stream {:?}", e);
                    this.reconnect_fut = Some(Box::pin(this.exchange.clone().make_owned_ws_connection()));
                    return this.handle_retry(e.normalized_with_exchange(T::EXCHANGE, None))
                }
                Poll::Pending => ()
            }
        } else {
            this.reconnect_fut = Some(Box::pin(this.exchange.clone().make_owned_ws_connection()));

            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        Poll::Pending
    }
}
