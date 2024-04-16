#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NormalizedTradingPair {
    pub base: String,
    pub quote: String,
}
