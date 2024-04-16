use crate::exchanges::Exchange;

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
