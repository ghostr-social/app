use anyhow::Result;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct DeleteClient(Client);

impl MediaHttpRequests for DeleteClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        Ok(self.0.delete(raw_url))
    }
}

#[tokio::test]
async fn media_get_overwrites_an_injected_request_method() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/video", listener.local_addr().unwrap());
    let observed = tokio::spawn(observe_request(listener));
    let requests = MediaRequestExecutor::new(
        Arc::new(DeleteClient(Client::new())),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );

    let response = requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await
        .unwrap();

    assert!(observed.await.unwrap().starts_with("GET /video HTTP/1.1"));
    drop(response);
}

async fn observe_request(listener: TcpListener) -> String {
    let (mut socket, _) = listener.accept().await.unwrap();
    let mut request = [0_u8; 4096];
    let count = socket.read(&mut request).await.unwrap();
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .await
        .unwrap();
    String::from_utf8(request[..count].to_vec()).unwrap()
}
