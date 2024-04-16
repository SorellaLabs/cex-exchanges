pub mod errors;
pub mod mutli;

use std::{
    pin::Pin,
    task::{Context, Poll}
};

use futures::{Future, FutureExt, SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::errors::WsError;
use crate::{
    exchanges::Exchange,
    types::normalized::ws::{combined::CombinedWsMessage, MessageOrPing}
};

type ReconnectFuture = Option<Pin<Box<dyn Future<Output = Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>>>>>;

type StreamConn = Pin<Box<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

pub struct WsStream<T> {
    exchange:      T,
    stream:        Option<StreamConn>,
    reconnect_fut: ReconnectFuture
}

impl<T> WsStream<T>
where
    T: Exchange
{
    pub fn new(exchange: T) -> Self {
        Self { exchange, stream: None, reconnect_fut: None }
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        let ws = self.exchange.make_ws_connection().await?;
        self.stream = Some(Box::pin(ws));
        Ok(())
    }

    fn handle_incoming(message: Message) -> Result<MessageOrPing<T>, WsError> {
        match message {
            Message::Text(msg) => {
                println!("MSG: {}", msg);
                let val = serde_json::from_str(&msg);

                Ok(MessageOrPing::new_message(val?))
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
    T: Exchange + Send + Unpin + 'static
{
    type Item = CombinedWsMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(stream) = this.stream.as_mut() {
            if let Poll::Ready(val) = stream.poll_next_unpin(cx) {
                let ret_val = match val {
                    Some(Ok(msg)) => match Self::handle_incoming(msg) {
                        Ok(MessageOrPing::Message(d)) => d.into(),
                        Ok(MessageOrPing::Ping) => {
                            if let Err(e) = Self::flush_sink_queue(stream, cx) {
                                this.stream = None;
                                return Poll::Ready(Some(e.normalized_with_exchange(T::EXCHANGE)));
                            } else if let Err(e) = stream.start_send_unpin(Message::Pong(vec![])) {
                                this.stream = None;
                                return Poll::Ready(Some(WsError::StreamTxError(e).normalized_with_exchange(T::EXCHANGE)));
                            }

                            return Poll::Pending;
                        }
                        Err(e) => {
                            this.stream = None;
                            e.normalized_with_exchange(T::EXCHANGE)
                        }
                    },
                    Some(Err(e)) => {
                        this.stream = None;
                        WsError::StreamRxError(e).normalized_with_exchange(T::EXCHANGE)
                    }
                    None => {
                        this.stream = None;
                        WsError::StreamTerminated.normalized_with_exchange(T::EXCHANGE)
                    }
                };

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
                    return Poll::Ready(Some(e.normalized_with_exchange(T::EXCHANGE)));
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
