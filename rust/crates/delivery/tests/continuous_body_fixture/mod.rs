mod body;
mod response;

use axum::routing::get;
use axum::Router;
use core::sync::atomic::AtomicUsize;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;

pub const TOTAL: u64 = 16 * 1024 * 1024;
pub struct Origin {
    pub url: String,
    pub whole_requests: Arc<AtomicUsize>,
}

pub fn harness(prefix: &str) -> super::delivery_fixture::DeliveryHarness {
    super::delivery_fixture::start_harness(
        prefix,
        super::delivery_fixture::options::production_geometry_parallel_options(),
    )
}

pub async fn serve(last_modified: bool) -> Origin {
    let route = if last_modified {
        get(response::dated_answer)
    } else {
        get(response::answer)
    };
    serve_route(route).await
}

async fn serve_route(route: axum::routing::MethodRouter<Arc<AtomicUsize>>) -> Origin {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("origin");
    let url = format!(
        "http://{}/video.mp4",
        listener.local_addr().expect("address")
    );
    let whole_requests = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/video.mp4", route)
        .with_state(Arc::clone(&whole_requests));
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve origin") });
    Origin {
        url,
        whole_requests,
    }
}

pub async fn wait_at(store: &PartialRangeStore, offset: u64) {
    loop {
        if store
            .read_range("current", offset..offset + 4)
            .await
            .expect("read")
            .as_deref()
            == Some(&[7; 4])
        {
            return;
        }
        tokio::time::sleep(core::time::Duration::from_millis(10)).await;
    }
}
