use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll}
};

use futures::{Future, FutureExt, SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

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
    msg_count:     u64
}

impl<T> WsStream<T>
where
    T: Exchange + Send
{
    pub fn new(exchange: T) -> Self {
        Self { exchange, stream: None, reconnect_fut: None, msg_count: 0 }
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        let ws = self.exchange.make_ws_connection().await;
        if let Err(e) = ws {
            println!("ERROR: {:?}", e);
            return Err(e)
        }

        self.stream = Some(Box::pin(ws?));
        Ok(())
    }

    fn handle_incoming(message: Message) -> Result<MessageOrPing<T>, (WsError, String)> {
        match message {
            Message::Text(msg) => {
                // println!("MSG: {}", msg);

                let mut des_msg = serde_json::from_str::<T::WsMessage>(&msg).map_err(|e| (e.into(), msg.clone()))?;
                des_msg.make_critical(msg);
                Ok(MessageOrPing::new_message(des_msg))
            }
            Message::Ping(_) => Ok(MessageOrPing::new_ping()),
            _ => unimplemented!()
        }
    }

    fn flush_sink_queue(stream: &mut StreamConn, cx: &mut Context<'_>) -> Result<(), WsError> {
        loop {
            match stream.poll_ready_unpin(cx) {
                Poll::Ready(Ok(_)) => return Ok(()),
                Poll::Ready(Err(e)) => return Err(WsError::StreamTxError(e)),
                Poll::Pending => ()
            }
        }
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
                let mut ret_val = match val {
                    Some(Ok(msg)) => match Self::handle_incoming(msg) {
                        Ok(MessageOrPing::Message(d)) => d.into(),
                        Ok(MessageOrPing::Ping) => {
                            if let Err(e) = Self::flush_sink_queue(stream, cx) {
                                this.stream = None;
                                return Poll::Ready(Some(e.normalized_with_exchange(T::EXCHANGE, None)));
                            } else if let Err(e) = stream.start_send_unpin(Message::Pong(vec![])) {
                                this.stream = None;
                                return Poll::Ready(Some(WsError::StreamTxError(e).normalized_with_exchange(T::EXCHANGE, None)));
                            }

                            return Poll::Pending;
                        }
                        Err((e, raw_msg)) => {
                            this.stream = None;
                            e.normalized_with_exchange(T::EXCHANGE, Some(raw_msg))
                        }
                    },
                    Some(Err(e)) => {
                        this.stream = None;
                        WsError::StreamRxError(e).normalized_with_exchange(T::EXCHANGE, None)
                    }
                    None => {
                        this.stream = None;

                        WsError::StreamTerminated.normalized_with_exchange(T::EXCHANGE, None)
                    }
                };

                if let Some(p) = ret_val.bad_pair() {
                    this.exchange.remove_bad_pair(p.clone());
                }

                this.msg_count += 1;
                return Poll::Ready(Some(ret_val));
            }
        } else if let Some(reconnect) = this.reconnect_fut.as_mut() {
            match reconnect.poll_unpin(cx) {
                Poll::Ready(Ok(new_stream)) => {
                    this.stream = Some(Box::pin(new_stream));
                    this.reconnect_fut = None;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    this.reconnect_fut = Some(Box::pin(this.exchange.clone().make_owned_ws_connection()));
                    return Poll::Ready(Some(e.normalized_with_exchange(T::EXCHANGE, None)));
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
