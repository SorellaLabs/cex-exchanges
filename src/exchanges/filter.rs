pub trait ExchangeFilter<T> {
    fn matches(&self, val: &T) -> bool;

    fn filter_matches(&self, vals: &mut Vec<T>) {
        vals.retain(|val| self.matches(val));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EmptyFilter;

impl<T> ExchangeFilter<T> for EmptyFilter {
    fn matches(&self, _val: &T) -> bool {
        true
    }
}

pub trait IterExchangeFilter<T>: IntoIterator
where
    <Self as IntoIterator>::Item: ExchangeFilter<T>
{
    fn matches_all(&self, val: &T) -> bool;

    fn matches_any(&self, val: &T) -> bool;
}

/// this implementation retains values that match ANY of the filters
impl<T, D> ExchangeFilter<T> for Vec<D>
where
    D: ExchangeFilter<T>
{
    fn matches(&self, val: &T) -> bool {
        self.iter().any(|v| v.matches(val))
    }
}
