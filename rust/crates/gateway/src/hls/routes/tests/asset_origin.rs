use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

const MANIFEST: &str = "#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";

pub(super) async fn serve_asset(response: Vec<u8>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut root, root_request) = accept(&listener).await;
        write_manifest(&mut root).await;
        drop(root);
        let (mut asset, asset_request) = accept(&listener).await;
        asset.write_all(&response).await.expect("asset response");
        vec![root_request, asset_request]
    });
    (source, task)
}

pub(super) async fn serve_optional_asset(response: Vec<u8>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut root, root_request) = accept(&listener).await;
        write_manifest(&mut root).await;
        drop(root);
        let mut requests = vec![root_request];
        if let Ok((mut asset, request)) =
            timeout(Duration::from_millis(150), accept(&listener)).await
        {
            asset.write_all(&response).await.expect("asset response");
            requests.push(request);
        }
        requests
    });
    (source, task)
}

pub(super) async fn serve_stalled_asset() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let source = format!("http://{}/index.m3u8", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut root, _) = accept(&listener).await;
        write_manifest(&mut root).await;
        drop(root);
        let (mut asset, _) = accept(&listener).await;
        asset
            .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n")
            .await
            .expect("asset headers");
        let mut byte = [0];
        assert_eq!(asset.read(&mut byte).await.expect("asset close"), 0);
    });
    (source, task)
}

pub(super) async fn accept(listener: &TcpListener) -> (TcpStream, String) {
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

pub(super) async fn write_manifest(socket: &mut TcpStream) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        MANIFEST.len(),
        MANIFEST
    );
    socket
        .write_all(response.as_bytes())
        .await
        .expect("manifest");
}
