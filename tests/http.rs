#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{clients::http::ExchangeApi, exchanges::coinbase::Coinbase};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_currencies() {
        let exchange_api = ExchangeApi::new();
        let all_currencies = exchange_api.all_symbols::<Coinbase>().await;
        assert!(all_currencies.is_ok());

        let all_currencies = all_currencies.unwrap();

        #[cfg(feature = "test-utils")]
        assert!(cex_exchanges::types::test_utils::NormalizedEquals::equals_normalized(all_currencies))
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod binance_tests {
    use cex_exchanges::{clients::http::ExchangeApi, exchanges::binance::Binance};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_symbols() {
        let exchange_api = ExchangeApi::new();
        let all_symbols = exchange_api.all_symbols::<Binance>().await;
        assert!(all_symbols.is_ok());

        let all_symbols = all_symbols.unwrap();

        #[cfg(feature = "test-utils")]
        assert!(cex_exchanges::types::test_utils::NormalizedEquals::equals_normalized(all_symbols))
    }
}
