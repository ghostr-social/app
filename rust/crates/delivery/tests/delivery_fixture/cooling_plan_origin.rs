use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

mod response;

#[derive(Clone)]
struct OriginState {
    failures: Arc<AtomicUsize>,
    useful: Arc<AtomicUsize>,
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

pub struct CoolingPlanOrigin {
    base: String,
    state: OriginState,
}

impl CoolingPlanOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let state = OriginState {
            failures: Arc::new(AtomicUsize::new(0)),
            useful: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
        };
        let app = Router::new()
            .route("/{kind}", get(response::media).head(response::media))
            .with_state(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("valid test fixture");
        });
        Self {
            base: format!("http://{address}"),
            state,
        }
    }

    pub fn url(&self, kind: &str) -> String {
        format!("{}/{kind}", self.base)
    }

    pub fn failures(&self) -> usize {
        self.state.failures.load(Ordering::SeqCst)
    }

    pub fn useful_requests(&self) -> usize {
        self.state.useful.load(Ordering::SeqCst)
    }

    pub async fn wait_useful(&self) {
        self.state
            .started
            .acquire()
            .await
            .expect("valid test fixture")
            .forget();
    }

    pub fn release(&self) {
        self.state.release.add_permits(4);
    }
}
