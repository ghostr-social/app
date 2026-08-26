//! Cooperative cancellation for chunk transfers after scroll-past.
//!
//! The engine keeps the [`CancelHandle`], and the downloader polls the [`CancelToken`]. A watch
//! channel provides the link without another dependency.

use tokio::sync::watch;

/// Creates a linked pair: keep the handle, hand the token to the
/// transfer being supervised.
pub fn cancel_pair() -> (CancelHandle, CancelToken) {
    let (sender, receiver) = watch::channel(false);
    (CancelHandle { sender }, CancelToken { receiver })
}

/// The canceller's side. Dropping it without calling
/// [`CancelHandle::cancel`] leaves the token uncancelled forever.
#[derive(Debug)]
pub struct CancelHandle {
    sender: watch::Sender<bool>,
}

impl CancelHandle {
    pub fn cancel(&self) {
        let _ = self.sender.send(true);
    }
}

/// The transfer's side; cheap to clone.
#[derive(Clone, Debug)]
pub struct CancelToken {
    receiver: watch::Receiver<bool>,
}

impl CancelToken {
    pub fn is_cancelled(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Resolves once the handle cancels; never resolves if the handle
    /// is dropped without cancelling (mirrors tokio-util semantics).
    pub async fn cancelled(&self) {
        let mut receiver = self.receiver.clone();
        if receiver.wait_for(|cancelled| *cancelled).await.is_err() {
            core::future::pending::<()>().await;
        }
    }
}
