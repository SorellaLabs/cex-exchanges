use cex_exchanges::exchanges::Exchange;
use cex_exchanges::ws::WsStream;
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
        if i == iterations {
            break;
        }
        i += 1;
    }
}
