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
