use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

const FAILURE: &[u8] =
    b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

pub struct Origin {
    pub url: String,
    methods: Arc<Mutex<Vec<String>>>,
}

pub async fn serve() -> Origin {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let methods = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&methods);
    tokio::spawn(async move {
        let (mut head, _) = listener.accept().await.expect("HEAD request");
        record(&observed, read_method(&mut head).await);
        let (mut failed, _) = listener.accept().await.expect("failed body request");
        record(&observed, read_method(&mut failed).await);
        failed.write_all(FAILURE).await.expect("failure response");
        failed.shutdown().await.expect("close failure response");
        let (mut retry, _) = listener.accept().await.expect("retry request");
        record(&observed, read_method(&mut retry).await);
    });
    Origin {
        url: format!("http://{address}/video.mp4"),
        methods,
    }
}

impl Origin {
    pub fn methods(&self) -> Vec<String> {
        self.methods.lock().expect("method log").clone()
    }
}

fn record(methods: &Mutex<Vec<String>>, method: String) {
    methods.lock().expect("method log").push(method);
}

async fn read_method(socket: &mut TcpStream) -> String {
    let mut request = Vec::new();
    while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
        let mut buffer = [0; 1_024];
        let read = socket.read(&mut buffer).await.expect("request headers");
        assert!(read > 0, "request ended before headers");
        request.extend_from_slice(&buffer[..read]);
    }
    let line_end = request
        .windows(2)
        .position(|bytes| bytes == b"\r\n")
        .expect("request line");
    String::from_utf8_lossy(&request[..line_end])
        .split_whitespace()
        .next()
        .expect("request method")
        .to_owned()
}
