use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub(super) async fn stalled_headers() -> (String, JoinHandle<()>) {
    let (listener, url) = listener().await;
    let task = tokio::spawn(async move {
        let _socket = accept(&listener).await;
        std::future::pending::<()>().await;
    });
    (url, task)
}

pub(super) async fn stalled_manifest_body() -> (String, JoinHandle<()>) {
    let (listener, url) = listener().await;
    let task = tokio::spawn(async move {
        let mut socket = accept(&listener).await;
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 100\r\n\r\n#EXTM3U\n")
            .await
            .expect("manifest prefix");
        std::future::pending::<()>().await;
    });
    (url, task)
}

pub(super) async fn oversized_manifest_headers() -> (String, JoinHandle<()>) {
    let (listener, url) = listener().await;
    let task = tokio::spawn(async move {
        let mut socket = accept(&listener).await;
        let padding = "a".repeat(33 * 1024);
        let response = format!(
            "HTTP/1.1 200 OK\r\nX-Padding: {padding}\r\nContent-Length: 8\r\n\r\n#EXTM3U\n"
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("response");
    });
    (url, task)
}

pub(super) async fn partial_manifest() -> (String, JoinHandle<()>) {
    let (listener, url) = listener().await;
    let task = tokio::spawn(async move {
        let mut socket = accept(&listener).await;
        let manifest = b"#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
        let response = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\nContent-Range: bytes 0-{}/{}\r\nConnection: close\r\n\r\n",
            manifest.len(),
            manifest.len() - 1,
            manifest.len()
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("headers");
        socket.write_all(manifest).await.expect("manifest");
    });
    (url, task)
}

pub(super) async fn manifest_then_stalled_asset() -> (String, JoinHandle<()>) {
    let (listener, url) = listener().await;
    let task = tokio::spawn(async move {
        let mut root = accept(&listener).await;
        let manifest = b"#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", manifest.len());
        root.write_all(header.as_bytes()).await.expect("headers");
        root.write_all(manifest).await.expect("manifest");
        drop(root);
        let mut asset = accept(&listener).await;
        asset
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nx")
            .await
            .expect("asset prefix");
        std::future::pending::<()>().await;
    });
    (url, task)
}

async fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let url = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    (listener, url)
}

async fn accept(listener: &TcpListener) -> TcpStream {
    let (mut socket, _) = listener.accept().await.expect("request");
    let mut request = [0; 1024];
    let read = socket.read(&mut request).await.expect("read request");
    assert!(read > 0, "request closed before sending bytes");
    socket
}
