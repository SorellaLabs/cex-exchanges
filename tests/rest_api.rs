#[cfg(feature = "us")]
#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{clients::rest_api::ExchangeApi, coinbase::Coinbase};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_currencies() {
        let exchange_api = ExchangeApi::new();
        let all_currencies = exchange_api.all_currencies::<Coinbase>().await;
        all_currencies.as_ref().unwrap();
        assert!(all_currencies.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_currencies = all_currencies.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_currencies));
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Coinbase>().await;
        assert!(all_instruments.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_instruments = all_instruments.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_instruments));
        }
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod binance_tests {
    use cex_exchanges::{binance::Binance, clients::rest_api::ExchangeApi};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_symbols() {
        let exchange_api = ExchangeApi::new();
        let all_symbols = exchange_api.all_currencies::<Binance>().await;
        assert!(all_symbols.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_symbols = all_symbols.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_symbols));
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Binance>().await;
        assert!(all_instruments.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_instruments = all_instruments.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_instruments));
        }
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod okex_tests {
    use cex_exchanges::{clients::rest_api::ExchangeApi, okex::Okex};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_symbols() {
        let exchange_api = ExchangeApi::new();
        let all_symbols = exchange_api.all_currencies::<Okex>().await;
        assert!(all_symbols.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_symbols = all_symbols.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_symbols));
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Okex>().await;
        all_instruments.as_ref().unwrap();
        assert!(all_instruments.is_ok());

        #[cfg(feature = "test-utils")]
        {
            let all_instruments = all_instruments.unwrap();
            assert!(cex_exchanges::exchanges::test_utils::NormalizedEquals::equals_normalized(all_instruments));
        }
    }
}
