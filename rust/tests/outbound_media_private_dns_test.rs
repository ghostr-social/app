use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct StaticResolver(SocketAddr);

impl Resolve for StaticResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        let addresses: Addrs = Box::new(vec![self.0].into_iter());
        Box::pin(async move { Ok(addresses) })
    }
}

#[tokio::test]
async fn rejects_a_hostname_resolving_to_a_private_address() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        let bytes = socket.read(&mut request).await.expect("read request");
        assert!(bytes > 0, "empty request");
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .expect("response");
    });
    let client =
        MediaHttpClient::with_resolver(Arc::new(StaticResolver(address))).expect("media client");

    let result = tokio::time::timeout(
        Duration::from_secs(1),
        client
            .get("http://media.test/video.mp4")
            .expect("request")
            .send(),
    )
    .await
    .expect("request completed");

    assert!(result.is_err());
    assert!(!server.is_finished(), "private server received the request");
    server.abort();
}
