use crate::{exchanges::Exchange, types::coinbase::messages::CoinbaseWsMessage};

#[derive(Debug, Clone)]
pub enum NormalizedWsMessage {
    Coinbase(CoinbaseWsMessage),
    Disconnect { exchange: String, message: String }
}

impl NormalizedWsMessage {
    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }

    pub fn is_err(&self) -> bool {
        matches!(self, NormalizedWsMessage::Disconnect { .. })
    }
}

macro_rules! normalized_ws {
    ($exchange:ident) => {
        paste::paste! {
            impl From<[<$exchange WsMessage>]> for NormalizedWsMessage {
                fn from(value: [<$exchange WsMessage>]) -> Self {
                    Self::$exchange(value)
                }
            }
        }
    };
}

normalized_ws!(Coinbase);

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
