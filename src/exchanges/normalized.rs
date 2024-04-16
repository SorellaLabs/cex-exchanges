#[derive(Debug, Clone)]
pub struct NormalizedExchange {
    pub exchange: CexExchange,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CexExchange {
    Coinbase,
}

impl ToString for CexExchange {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
