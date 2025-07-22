use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Future, FutureExt, SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, trace, warn};

use super::{WsError, WsStreamConfig};
use crate::{
    clients::ws::critical::CriticalWsMessage,
    exchanges::normalized::ws::{CombinedWsMessage, MessageOrPing},
    Exchange,
};

type StreamConn = Pin<Box<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

pub struct WsStream<T> {
    exchange: T,
    stream: Option<StreamConn>,
    stream_futs: WsStreamFutures<T>,
    config: WsStreamConfig,
    retry_count: u64,
}

impl<T> WsStream<T>
where
    T: Exchange,
{
    pub fn new(exchange: T, config: WsStreamConfig) -> Self {
        Self { stream_futs: WsStreamFutures::new(exchange.clone()), exchange, stream: None, config, retry_count: 0 }
    }

    pub fn exchange(&self) -> T {
        self.exchange.clone()
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        let ws = self.exchange.make_ws_connection().await.inspect_err(|e| {
            error!(target: "cex-exchanges::live-stream",  exchange=?T::EXCHANGE, "error connecting to the websocket stream: {:?}", e);
        })?;

        self.stream = Some(Box::pin(ws));
        Ok(())
    }

    fn reconnect(&mut self, cx: &mut Context<'_>) {
        self.stream = None;
        self.stream_futs.new_reconnect();
        cx.waker().wake_by_ref();
    }

    fn handle_incoming(message: Message) -> Result<MessageOrPing<T>, (WsError, String)> {
        match message {
            Message::Text(msg) => {
                trace!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "recieved new message for the stream: {}", msg);
                let mut des_msg = serde_json::from_str::<T::WsMessage>(&msg).map_err(|e| (e.into(), msg.clone()))?;
                des_msg.make_critical(msg);
                Ok(MessageOrPing::new_message(des_msg))
            }
            Message::Ping(ping) => Ok(MessageOrPing::new_ping(ping)),
            Message::Binary(_) => panic!("Exchange: {} - Message::Binary", T::EXCHANGE),
            Message::Pong(_) => panic!("Exchange: {} - Message::Pong", T::EXCHANGE),
            Message::Close(_) => Ok(MessageOrPing::new_close()),
            Message::Frame(_) => panic!("Exchange: {} - Message::Frame", T::EXCHANGE),
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
                Poll::Pending => (),
            }
        }

        debug!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "finished flushing queue sink with a PONG send");

        ret
    }

    fn handle_retry(&mut self, msg: CombinedWsMessage) -> Poll<Option<CombinedWsMessage>> {
        let stay_same = match self.handle_bad_pair(&msg) {
            Some(true) => return Poll::Ready(None),
            Some(false) => false,
            None => true,
        };
        if let Some(retries) = self.config.max_retries {
            if !stay_same {
                self.retry_count += 1;
            }

            if self.retry_count > retries {
                error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "retries exceeded -- EXITING");
                return Poll::Ready(None);
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

    #[allow(unused)]
    fn sanity_check(&self) {
        if self.stream_futs.is_reconnecting() {
            assert!(self.stream_futs.timeout_rx.is_none() && self.stream.is_none());
        }

        if self.stream.is_some() {
            assert!(!self.stream_futs.is_reconnecting() && self.stream_futs.timeout_rx.is_some());
        }
    }
}

impl<T> Stream for WsStream<T>
where
    T: Exchange + Debug,
    Self: Send,
{
    type Item = CombinedWsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // this.sanity_check();

        if let Some(stream) = this.stream.as_mut() {
            if let Poll::Ready(val) = stream.poll_next_unpin(cx) {
                match val {
                    Some(Ok(msg)) => match Self::handle_incoming(msg) {
                        Ok(MessageOrPing::Message(d)) => {
                            this.stream_futs.new_timeout_rx();
                            return this.handle_retry(d.into());
                        }
                        Ok(MessageOrPing::Ping(v)) => {
                            debug!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "recieved ping");
                            if let Err(e) = stream.start_send_unpin(Message::Pong(v)) {
                                error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error sending pong");
                                this.reconnect(cx);
                                return this.handle_retry(WsError::StreamTxError(e).normalized_with_exchange(T::EXCHANGE, None));
                            } else if let Err(e) = Self::flush_sink_queue(stream, cx) {
                                warn!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error flushing queue sink");
                                this.reconnect(cx);
                                return this.handle_retry(e.normalized_with_exchange(T::EXCHANGE, None));
                            }
                        }
                        Ok(MessageOrPing::Close) => {
                            this.reconnect(cx);
                            return Poll::Pending;
                        }
                        Err((e, raw_msg)) => {
                            this.reconnect(cx);
                            return this.handle_retry(e.normalized_with_exchange(T::EXCHANGE, Some(raw_msg)));
                        }
                    },
                    Some(Err(e)) => {
                        this.reconnect(cx);
                        return this.handle_retry(WsError::StreamRxError(e).normalized_with_exchange(T::EXCHANGE, None));
                    }
                    None => {
                        this.reconnect(cx);
                        return this.handle_retry(WsError::StreamTerminated.normalized_with_exchange(T::EXCHANGE, None));
                    }
                };
            }

            if this.stream_futs.is_timed_out(cx) {
                this.reconnect(cx);
            }
        } else if let Poll::Ready(Some(stream_futs)) = this.stream_futs.poll_next_unpin(cx) {
            match stream_futs {
                Ok(new_stream) => {
                    this.stream_futs.new_timeout_rx();
                    this.stream = Some(Box::pin(new_stream));
                }
                Err(err) => {
                    this.stream_futs.new_reconnect();
                    return this.handle_retry(err.normalized_with_exchange(T::EXCHANGE, None));
                }
            }
            cx.waker().wake_by_ref();
        } else if !this.stream_futs.is_reconnecting() {
            cx.waker().wake_by_ref();
            this.stream_futs.new_reconnect();
        }

        Poll::Pending
    }
}

type ReconnectFuture = Option<Pin<Box<dyn Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>> + Send + 'static>>>;
type TimeoutRxFuture = Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>;

struct WsStreamFutures<T> {
    exchange: T,
    reconnect: ReconnectFuture,
    timeout_rx: TimeoutRxFuture,
}

impl<T: Exchange> WsStreamFutures<T> {
    fn new(exchange: T) -> Self {
        Self { exchange, reconnect: None, timeout_rx: None }
    }

    fn new_reconnect(&mut self) {
        self.reconnect = Some(Box::pin(self.exchange.clone().make_owned_ws_connection()));
        self.timeout_rx = None;
    }

    fn new_timeout_rx(&mut self) {
        if let Some(timeout_sec) = T::STREAM_TIMEOUT_MS {
            self.timeout_rx = Some(Box::pin(tokio::time::sleep(std::time::Duration::from_millis(timeout_sec))));
        } else {
            panic!()
        }
    }

    fn is_reconnecting(&self) -> bool {
        self.reconnect.is_some()
    }

    fn is_timed_out(&mut self, cx: &mut Context<'_>) -> bool {
        if let Some(mut timeout) = self.timeout_rx.take() {
            if timeout.poll_unpin(cx).is_ready() {
                warn!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "stream timed out - reconnecting");
                self.new_reconnect();
                return true;
            } else {
                self.timeout_rx = Some(timeout);
            }
        }

        false
    }
}

impl<T: Exchange> Stream for WsStreamFutures<T> {
    type Item = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(mut reconnect) = this.reconnect.take() {
            if let Poll::Ready(new_stream_res) = reconnect.poll_unpin(cx) {
                if let Err(e) = new_stream_res.as_ref() {
                    error!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "error reconnecting to stream {:?}", e);
                } else {
                    this.new_timeout_rx();
                    info!(target: "cex-exchanges::live-stream", exchange=?T::EXCHANGE, "successfully reconnected to stream");
                }
                return Poll::Ready(Some(new_stream_res));
            } else {
                this.reconnect = Some(reconnect);
            }
        }

        Poll::Pending
    }
}
