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

        {
            let all_currencies = all_currencies.unwrap();
            let test_length = all_currencies
                .clone()
                .take_coinbase_currencies()
                .unwrap()
                .len();
            assert!(test_length > 10);

            let normalized = all_currencies.clone().normalize();
            let test_length = normalized.clone().take_currencies().unwrap().len();
            assert!(test_length > 10);

            assert_eq!(all_currencies, normalized);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Coinbase>().await;
        all_instruments.as_ref().unwrap();
        assert!(all_instruments.is_ok());

        {
            let all_instruments = all_instruments.unwrap();
            let normalized = all_instruments.clone().normalize();
            assert_eq!(all_instruments, normalized);
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
        all_symbols.as_ref().unwrap();
        assert!(all_symbols.is_ok());

        {
            let all_symbols = all_symbols.unwrap();
            let test_length = all_symbols.clone().take_binance_currencies().unwrap().len();
            assert!(test_length > 10);

            let normalized = all_symbols.clone().normalize();
            let test_length = normalized.clone().take_currencies().unwrap().len();
            assert!(test_length > 10);

            assert_eq!(all_symbols, normalized);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Binance>().await;
        all_instruments.as_ref().unwrap();
        assert!(all_instruments.is_ok());

        {
            let all_instruments = all_instruments.unwrap();
            let test_length = all_instruments
                .clone()
                .take_binance_instruments()
                .unwrap()
                .len();
            assert!(test_length > 10);

            let normalized = all_instruments.clone().normalize();
            println!("{:?}", normalized);

            let test_length = normalized.clone().take_instruments().unwrap().len();
            assert!(test_length > 10);

            assert_eq!(all_instruments, normalized);
        }
    }
}

#[cfg(feature = "us")]
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

        {
            let all_symbols = all_symbols.unwrap();
            let test_length = all_symbols.clone().take_okex_currencies().unwrap().len();
            assert!(test_length > 10);

            let normalized = all_symbols.clone().normalize();
            let test_length = normalized.clone().take_currencies().unwrap().len();
            assert!(test_length > 10);

            assert_eq!(all_symbols, normalized);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Okex>().await;
        all_instruments.as_ref().unwrap();
        assert!(all_instruments.is_ok());

        {
            let all_instruments = all_instruments.unwrap();
            let test_length = all_instruments
                .clone()
                .take_okex_instruments()
                .unwrap()
                .len();
            assert!(test_length > 10);

            let normalized = all_instruments.clone().normalize();
            let test_length = normalized.clone().take_instruments().unwrap().len();
            assert!(test_length > 10);
        }
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod kucoin_tests {
    use cex_exchanges::{clients::rest_api::ExchangeApi, kucoin::Kucoin};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_all_currencies() {
        let exchange_api = ExchangeApi::new();
        let all_currencies = exchange_api.all_currencies::<Kucoin>().await;
        all_currencies.as_ref().unwrap();
        assert!(all_currencies.is_ok());

        {
            let all_currencies = all_currencies.unwrap();
            let test_length = all_currencies
                .clone()
                .take_kucoin_currencies()
                .unwrap()
                .len();
            assert!(test_length > 10);

            let normalized = all_currencies.clone().normalize();
            let test_length = normalized.clone().take_currencies().unwrap().len();
            assert!(test_length > 10);

            assert_eq!(all_currencies, normalized);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_all_symbols() {
        let exchange_api = ExchangeApi::new();
        let all_symbols = exchange_api.all_instruments::<Kucoin>().await;
        all_symbols.as_ref().unwrap();
        assert!(all_symbols.is_ok());

        {
            let all_symbols = all_symbols.unwrap();
            let test_length = all_symbols.clone().take_kucoin_instruments().unwrap().len();
            assert!(test_length > 10);

            let normalized = all_symbols.clone().normalize();
            let test_length = normalized.clone().take_instruments().unwrap().len();
            assert!(test_length > 10);
        }
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod bybit_tests {
    use cex_exchanges::{bybit::Bybit, clients::rest_api::ExchangeApi};
    use serial_test::serial;

    // #[tokio::test]
    // #[serial]
    // async fn test_all_coins() {
    //     let exchange_api = ExchangeApi::new();
    //     let all_coins = exchange_api.all_currencies::<Bybit>().await;
    //     all_coins.as_ref().unwrap();
    //     assert!(all_coins.is_ok());

    //     {
    //         let all_coins = all_coins.unwrap();
    //         let test_length =
    // all_coins.clone().take_bybit_currencies().unwrap().len();         assert!
    // (test_length > 10);

    //         let normalized = all_coins.clone().normalize();
    //         let test_length =
    // normalized.clone().take_currencies().unwrap().len();         assert!
    // (test_length > 10);

    //         assert_eq!(all_coins, normalized);
    //     }
    // }

    #[tokio::test]
    #[serial]
    async fn test_all_instruments() {
        let exchange_api = ExchangeApi::new();
        let all_instruments = exchange_api.all_instruments::<Bybit>().await;
        all_instruments.as_ref().unwrap();
        assert!(all_instruments.is_ok());

        {
            let all_instruments = all_instruments.unwrap();
            let test_length = all_instruments
                .clone()
                .take_bybit_instruments()
                .unwrap()
                .len();
            assert!(test_length > 10);

            let normalized = all_instruments.clone().normalize();
            let test_length = normalized.clone().take_instruments().unwrap().len();
            assert!(test_length > 10);
        }
    }
}
