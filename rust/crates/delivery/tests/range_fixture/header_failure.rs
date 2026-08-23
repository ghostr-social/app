use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSink, ChunkSpec, DownloadTraffic,
    OpenedResponse,
};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn download(
    response: &'static [u8],
    prefix: &str,
) -> (anyhow::Result<ChunkResult>, Vec<OpenedResponse>, usize) {
    let (url, request) = serve(response).await;
    let client = super::media_client();
    let root = super::temp_root(prefix);
    let store = super::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let mut traffic = Headers::default();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: super::range_request(ByteRange::new(0, 8)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: Default::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = super::network();
    let result = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &token,
            network: &network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await;
    request.await.unwrap();
    std::fs::remove_dir_all(root).ok();
    (result, traffic.responses, traffic.opened)
}

async fn serve(response: &'static [u8]) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let request = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).await.unwrap() > 0);
        socket.write_all(response).await.unwrap();
    });
    (format!("http://{address}/video.mp4"), request)
}

#[derive(Default)]
struct Headers {
    responses: Vec<OpenedResponse>,
    opened: usize,
}

impl DownloadTraffic for Headers {
    fn opened(&mut self, _: Duration) {
        self.opened += 1;
    }
    fn wrote(&mut self, _: u64) {}
    fn response_observed(&mut self, response: OpenedResponse) {
        self.responses.push(response);
    }
}
