use axum::body::Bytes;
use axum::routing::get;
use axum::Router;
use core::convert::Infallible;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod server;

pub(super) type VideoState = (u64, mpsc::Sender<ActiveBody>);

pub struct VideoOrigin {
    pub url: String,
    bodies: mpsc::Receiver<ActiveBody>,
}

pub struct ActiveBody {
    pub(super) length: usize,
    pub(super) body: mpsc::Sender<Result<Bytes, Infallible>>,
}

pub struct ManifestOrigin {
    pub url: String,
    hits: mpsc::Receiver<()>,
}

impl VideoOrigin {
    pub async fn start(total: u64) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let (body, bodies) = mpsc::channel(2);
        let app = Router::new()
            .route("/video.mp4", get(server::video).head(server::video))
            .with_state((total, body));
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("valid test fixture");
        });
        Self {
            url: format!("http://{address}/video.mp4"),
            bodies,
        }
    }

    pub async fn next(&mut self) -> ActiveBody {
        self.bodies.recv().await.expect("video body request")
    }
}

impl ActiveBody {
    pub async fn finish(self) {
        let bytes = Bytes::from(vec![7; self.length]);
        self.body.send(Ok(bytes)).await.expect("body consumer");
    }
}

impl ManifestOrigin {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let (hit, hits) = mpsc::channel(2);
        let app = Router::new()
            .route("/index.m3u8", get(server::manifest))
            .with_state(hit);
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("valid test fixture");
        });
        Self {
            url: format!("http://{address}/index.m3u8"),
            hits,
        }
    }

    pub async fn next(&mut self) {
        self.hits.recv().await.expect("manifest request");
    }

    pub async fn expect_quiet(&mut self) {
        let hit =
            tokio::time::timeout(core::time::Duration::from_millis(75), self.hits.recv()).await;
        assert!(hit.is_err(), "HLS bypassed the occupied global gate");
    }
}
