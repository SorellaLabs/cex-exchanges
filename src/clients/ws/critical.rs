use std::fmt::Debug;

use serde::Deserialize;

use crate::normalized::ws::CombinedWsMessage;

pub trait CriticalWsMessage: for<'de> Deserialize<'de> + Into<CombinedWsMessage> + Debug {
    fn make_critical(&mut self, msg: String);
}
