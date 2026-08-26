use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

struct MappedClient(Client);

impl MediaHttpRequests for MappedClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

#[tokio::test]
async fn a_public_chain_cannot_pivot_into_an_adapter_allowed_loopback() {
    let target = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let target_address = target.local_addr().expect("valid test fixture");
    let target = tokio::spawn(async move {
        let (socket, _) = target.accept().await.expect("valid test fixture");
        answer(socket, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
    });
    let origin = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let origin_address = origin.local_addr().expect("valid test fixture");
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/private\r\nContent-Length: 0\r\n\r\n"
    );
    tokio::spawn(async move {
        let (socket, _) = origin.accept().await.expect("valid test fixture");
        answer(socket, redirect.as_bytes()).await;
    });
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve("public.example", origin_address)
        .build()
        .expect("valid test fixture");
    let executor = MediaRequestExecutor::new(
        Arc::new(MappedClient(client)),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );
    let url = format!("http://public.example:{}/start", origin_address.port());

    let result = executor
        .get(&url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await;

    assert!(result.is_err(), "public chain reached a private redirect");
    assert!(!target.is_finished(), "private target received the request");
    target.abort();
}

async fn answer(mut socket: TcpStream, response: &[u8]) {
    let mut request = [0u8; 2048];
    assert!(
        socket.read(&mut request).await.expect("valid test fixture") > 0,
        "test server should receive a request"
    );
    socket
        .write_all(response)
        .await
        .expect("valid test fixture");
}
