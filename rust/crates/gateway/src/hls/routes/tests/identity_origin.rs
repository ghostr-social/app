use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

const MANIFEST: &str = "#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";

pub(super) async fn coded_manifest() -> (String, JoinHandle<Vec<String>>) {
    let (listener, source) = listener().await;
    let task = tokio::spawn(async move {
        let (mut socket, request) = accept(&listener).await;
        write_manifest(&mut socket, true).await;
        vec![request]
    });
    (source, task)
}

pub(super) async fn manifest_then_coded_asset() -> (String, JoinHandle<Vec<String>>) {
    let (listener, source) = listener().await;
    let task = tokio::spawn(async move {
        let (mut root, root_request) = accept(&listener).await;
        write_manifest(&mut root, false).await;
        drop(root);
        let (mut asset, asset_request) = accept(&listener).await;
        asset
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: video/iso.segment\r\n\
                  Content-Length: 5\r\nContent-Encoding: identity\r\n\
                  Content-Encoding: gzip\r\nConnection: close\r\n\r\ncoded",
            )
            .await
            .expect("coded asset");
        vec![root_request, asset_request]
    });
    (source, task)
}

pub(super) fn assert_identity_request(request: &str) {
    let request = request.to_ascii_lowercase();
    let values: Vec<_> = request
        .lines()
        .filter_map(|line| line.strip_prefix("accept-encoding: "))
        .collect();
    assert_eq!(values, ["identity"]);
}

async fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    (listener, source)
}

async fn accept(listener: &TcpListener) -> (TcpStream, String) {
    let (mut socket, _) = listener.accept().await.expect("request");
    let mut request = Vec::new();
    loop {
        let mut chunk = [0; 1024];
        let read = socket.read(&mut chunk).await.expect("read request");
        assert!(read > 0, "request closed before headers");
        request.extend_from_slice(&chunk[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    (socket, String::from_utf8(request).expect("HTTP request"))
}

async fn write_manifest(socket: &mut TcpStream, coded: bool) {
    let encoding = coded.then_some("Content-Encoding: identity\r\nContent-Encoding: gzip\r\n");
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\n\
         Content-Length: {}\r\n{}Connection: close\r\n\r\n{}",
        MANIFEST.len(),
        encoding.unwrap_or_default(),
        MANIFEST
    );
    socket
        .write_all(response.as_bytes())
        .await
        .expect("manifest");
}
