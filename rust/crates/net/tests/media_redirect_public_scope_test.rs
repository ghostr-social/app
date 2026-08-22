use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct MappedClient(Client);

impl MediaHttpRequests for MappedClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

#[tokio::test]
async fn a_public_chain_cannot_pivot_into_an_adapter_allowed_loopback() {
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_address = target.local_addr().unwrap();
    let target = tokio::spawn(async move {
        let (socket, _) = target.accept().await.unwrap();
        answer(socket, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
    });
    let origin = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_address = origin.local_addr().unwrap();
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/private\r\nContent-Length: 0\r\n\r\n"
    );
    tokio::spawn(async move {
        let (socket, _) = origin.accept().await.unwrap();
        answer(socket, redirect.as_bytes()).await;
    });
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve("public.example", origin_address)
        .build()
        .unwrap();
    let executor = MediaRequestExecutor::new(
        Arc::new(MappedClient(client)),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );
    let url = format!("http://public.example:{}/start", origin_address.port());

    let result = executor
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await;

    assert!(result.is_err(), "public chain reached a private redirect");
    assert!(!target.is_finished(), "private target received the request");
    target.abort();
}

async fn answer(mut socket: TcpStream, response: &[u8]) {
    let mut request = [0u8; 2048];
    assert!(socket.read(&mut request).await.unwrap() > 0);
    socket.write_all(response).await.unwrap();
}
