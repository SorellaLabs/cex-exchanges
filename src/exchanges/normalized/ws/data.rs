use crate::{
    normalized::types::{NormalizedQuote, NormalizedTrade, NormalizedTradingPair},
    CexExchange, Exchange
};

#[derive(Debug, Clone)]
pub enum NormalizedWsDataTypes {
    Trade(NormalizedTrade),
    Trades(Vec<NormalizedTrade>),
    Quote(NormalizedQuote),
    Quotes(Vec<NormalizedQuote>),
    Disconnect { exchange: CexExchange, message: String, raw_message: String },
    RemovedPair { exchange: CexExchange, bad_pair: Option<NormalizedTradingPair>, raw_message: String },
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
