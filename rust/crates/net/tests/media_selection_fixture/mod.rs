use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits, MediaResponse};
use reqwest::header::HeaderMap;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;

pub async fn response(vary: &str, headers: HeaderMap) -> MediaResponse {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listen");
    let url = format!(
        "http://{}/video.mp4",
        listener.local_addr().expect("address")
    );
    let reply = format!("HTTP/1.1 206 Partial Content\r\nContent-Length: 1\r\nContent-Range: bytes 0-0/2\r\nETag: \"v1\"\r\nCache-Control: public, max-age=60\r\nVary: {vary}\r\nConnection: close\r\n\r\nx");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        read_headers(&mut socket).await;
        socket.write_all(reply.as_bytes()).await.expect("response");
    });
    let executor = MediaRequestExecutor::new(
        super::request_gate_fixture::LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("limits"),
    );
    let request = executor
        .get(&url, PreemptionAuthority::Transition)
        .expect("request");
    let request = headers.iter().fold(request, |request, (name, value)| {
        request.header(name.clone(), value.clone())
    });
    request
        .admit()
        .await
        .expect("admission")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(2),
        )
        .await
        .expect("origin response")
}

async fn read_headers(socket: &mut tokio::net::TcpStream) {
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") {
        assert!(request.len() < 4096, "bounded request head");
        request.push(socket.read_u8().await.expect("request head"));
    }
}

pub fn headers(range: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        "identity".parse().expect("encoding"),
    );
    headers.insert(reqwest::header::RANGE, range.parse().expect("range"));
    headers
}
