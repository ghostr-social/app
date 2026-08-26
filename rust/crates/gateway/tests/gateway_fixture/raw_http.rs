use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub async fn spawn_raw_server(response: &'static [u8]) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut buffer = vec![0; 4096];
        let bytes = socket.read(&mut buffer).await.expect("read request");
        socket.write_all(response).await.expect("write response");
        buffer.truncate(bytes);
        buffer
    });
    (format!("http://{address}/video.mp4"), request)
}

pub async fn unused_loopback_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    drop(listener);
    format!("http://{address}/video.mp4")
}

pub async fn spawn_response_sequence(responses: Vec<&'static [u8]>) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let requests = tokio::spawn(async move {
        for response in responses {
            let (mut socket, _) = listener.accept().await.expect("request");
            let mut buffer = vec![0; 4096];
            let bytes = socket.read(&mut buffer).await.expect("read request");
            assert!(bytes > 0, "empty request");
            socket.write_all(response).await.expect("write response");
        }
    });
    (format!("http://{address}/video.mp4"), requests)
}
