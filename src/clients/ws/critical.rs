use std::fmt::Debug;

use serde::Deserialize;

use crate::normalized::ws::CombinedWsMessage;

pub trait CriticalWsMessage: for<'de> Deserialize<'de> + Into<CombinedWsMessage> + Send + Debug {
    fn make_critical(&mut self, msg: String);
}
