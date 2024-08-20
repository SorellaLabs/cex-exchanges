use crate::{
    normalized::types::{NormalizedL2, NormalizedQuote, NormalizedTrade, NormalizedTradingPair},
    CexExchange, Exchange
};

#[derive(Debug, Clone)]
pub enum NormalizedWsDataTypes {
    Trade(NormalizedTrade),
    Trades(Vec<NormalizedTrade>),
    Quote(NormalizedQuote),
    Quotes(Vec<NormalizedQuote>),
    L2(NormalizedL2),
    Disconnect { exchange: CexExchange, message: String, raw_message: String },
    RemovedPair { exchange: CexExchange, bad_pair: NormalizedTradingPair, raw_message: String },
    Other { exchange: CexExchange, kind: String, value: String }
}

pub(crate) enum MessageOrPing<T: Exchange> {
    Message(T::WsMessage),
    Ping(Vec<u8>),
    Close
}

impl<T: Exchange> MessageOrPing<T> {
    pub(crate) fn new_message(msg: T::WsMessage) -> Self {
        MessageOrPing::Message(msg)
    }

    pub(crate) fn new_ping(ping: Vec<u8>) -> Self {
        MessageOrPing::Ping(ping)
    }

    pub(crate) fn new_close() -> Self {
        MessageOrPing::Close
    }
}
