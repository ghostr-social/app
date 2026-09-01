use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;

pub async fn serve(prefix: Vec<u8>, total: u64) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept");
        let mut request = vec![0; 4_096];
        let request_bytes = socket.read(&mut request).await.expect("request");
        assert!(request_bytes > 0, "request closed before sending bytes");
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n\
             Content-Type: video/mp4\r\nETag: \"promoted-stall\"\r\n\r\n"
        );
        socket.write_all(head.as_bytes()).await.expect("headers");
        socket.write_all(&prefix).await.expect("prefix");
        socket.flush().await.expect("flush");
        core::future::pending::<()>().await;
    });
    format!("http://{address}/video.mp4")
}
