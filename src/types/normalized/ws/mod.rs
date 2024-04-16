pub mod channels;
pub mod combined;
use super::{quotes::NormalizedQuote, trades::NormalizedTrade};
use crate::exchanges::{normalized::CexExchange, Exchange};

#[derive(Debug, Clone)]
pub enum NormalizedWsDataTypes {
    Trades(NormalizedTrade),
    Quotes(NormalizedQuote),
    Disconnect { exchange: CexExchange, message: String },
    Other { exchange: CexExchange, kind: String, value: String }
}

pub(crate) enum MessageOrPing<T: Exchange> {
    Message(T::WsMessage),
    Ping
}

impl<T: Exchange> MessageOrPing<T> {
    pub(crate) fn new_message(msg: T::WsMessage) -> Self {
        MessageOrPing::Message(msg)
    }

    pub(crate) fn new_ping() -> Self {
        MessageOrPing::Ping
    }
}
