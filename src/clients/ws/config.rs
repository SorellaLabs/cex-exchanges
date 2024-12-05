#[derive(Debug, Default, Copy, Clone)]
pub struct WsStreamConfig {
    pub max_retries: Option<u64>
}

impl WsStreamConfig {
    pub fn new(max_retries: Option<u64>) -> Self {
        Self { max_retries }
    }

    pub fn with_max_retries(mut self, max_retries: u64) -> Self {
        self.max_retries = Some(max_retries);
        self
    }
}
