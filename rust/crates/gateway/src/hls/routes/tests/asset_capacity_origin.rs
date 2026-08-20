use super::asset_origin::accept;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

const MANIFEST: &str = "#EXTM3U\n#EXTINF:2,\nfirst.m4s\n#EXTINF:2,\nsecond.m4s\n#EXT-X-ENDLIST\n";

pub(super) async fn serve() -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut root, _) = accept(&listener).await;
        write_manifest(&mut root).await;
        let mut requests = Vec::new();
        for _ in 0..2 {
            requests.push(write_range(&listener).await);
        }
        if let Ok(request) =
            tokio::time::timeout(Duration::from_millis(150), write_range(&listener)).await
        {
            requests.push(request);
        }
        requests
    });
    (source, task)
}

async fn write_manifest(socket: &mut TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        MANIFEST.len(), MANIFEST
    );
    socket
        .write_all(response.as_bytes())
        .await
        .expect("manifest");
}

async fn write_range(listener: &TcpListener) -> String {
    let (mut socket, request) = accept(listener).await;
    let start = if request.contains("bytes=4-7") { 4 } else { 0 };
    let body = if start == 0 { "abcd" } else { "efgh" };
    let response = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\nContent-Range: bytes {start}-{}/8\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n{body}",
        start + 3
    );
    socket.write_all(response.as_bytes()).await.expect("asset");
    request
}
