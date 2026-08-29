use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct HlsGate {
    pub started: Arc<Semaphore>,
    pub release: Arc<Semaphore>,
    hits: Arc<Mutex<Vec<&'static str>>>,
    blocked: &'static str,
}

impl HlsGate {
    pub fn new() -> Self {
        Self::blocking("root")
    }

    pub fn blocking(blocked: &'static str) -> Self {
        Self {
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
            hits: Arc::new(Mutex::new(Vec::new())),
            blocked,
        }
    }

    pub fn hits(&self) -> Vec<&'static str> {
        self.hits.lock().expect("valid test fixture").clone()
    }

    pub fn blocked(&self) -> &'static str {
        self.blocked
    }

    pub(super) async fn hit(&self, path: &'static str) {
        self.hits.lock().expect("valid test fixture").push(path);
        if path == self.blocked {
            self.started.add_permits(1);
            self.release
                .acquire()
                .await
                .expect("valid test fixture")
                .forget();
        }
    }
}
