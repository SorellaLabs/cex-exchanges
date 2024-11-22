use super::Exchange;

pub struct ExchangeBuilder;

impl ExchangeBuilder {
    pub fn with_exchange<E: Exchange>() -> ExchangeBuilderWithExchange<E> {
        ExchangeBuilderWithExchange { exchange: E::default() }
    }
}

pub struct ExchangeBuilderWithExchange<E> {
    exchange: E
}
