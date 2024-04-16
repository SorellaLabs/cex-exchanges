use cex_exchanges::{
    clients::ws::{mutli::MutliWsStreamBuilder, WsStream},
    exchanges::Exchange
};
use futures::StreamExt;

pub async fn stream_util<E: Exchange + Send + Unpin + 'static>(exchange: E, iterations: usize) {
    let mut stream = WsStream::new(exchange);
    stream.connect().await.unwrap();

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            println!("ERROR: {:?}", val);
        }

        assert!(val.is_ok());

        #[cfg(feature = "test-utils")]
        assert!(cex_exchanges::types::test_utils::NormalizedEquals::equals_normalized(val));

        if i == iterations {
            break;
        }
        i += 1;
    }
}

pub async fn mutlistream_util<E: Exchange + Send + Unpin + 'static>(builder: MutliWsStreamBuilder<E>, iterations: usize) {
    let mut stream = builder.build_multistream().await.unwrap();

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            println!("ERROR: {:?}", val);
        }

        assert!(val.is_ok());

        #[cfg(feature = "test-utils")]
        assert!(cex_exchanges::types::test_utils::NormalizedEquals::equals_normalized(val));

        if i == iterations {
            break;
        }
        i += 1;
    }
}
