#[path = "store_fixture/paused.rs"]
mod paused_fixture;
mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::future::{poll_fn, Future};
use std::path::Path;
use std::sync::Arc;
use std::task::Poll;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type Write = JoinHandle<anyhow::Result<bool>>;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn source_switch_fences_persisting_and_waiting_writers() {
    let mut fixture = paused_fixture::paused_store("partial-representation-write-race");
    let store = fixture.store.clone();
    let (binding, old, current) = identities();
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(old.clone()).await.unwrap();
    let persisting = persisting_write(store.clone(), old.clone());
    fixture.wait_until_admission().await;
    let (ready, waiting_ready) = oneshot::channel();
    let waiting = waiting_write(store.clone(), old, ready);
    waiting_ready.await.unwrap();

    let switching = tokio::spawn({
        let store = store.clone();
        async move { store.select_transfer(current).await }
    });
    fixture.resume();
    switching.await.unwrap().unwrap();

    assert_rejected(persisting, waiting).await;
    assert_discarded(&store, &fixture.root, &binding).await;
    store_fixture::discard(&fixture.root);
}

fn identities() -> (RepresentationBinding, TransferIdentity, TransferIdentity) {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post, mirrored_meta());
    let old = binding.transfer("https://a.example/video").unwrap();
    let current = binding.transfer("https://b.example/video").unwrap();
    (binding, old, current)
}

fn persisting_write(store: Arc<PartialRangeStore>, identity: TransferIdentity) -> Write {
    tokio::spawn(async move {
        store
            .write_range_for_transfer_if_current(&identity, 0, b"old")
            .await
    })
}

fn waiting_write(
    store: Arc<PartialRangeStore>,
    identity: TransferIdentity,
    ready: oneshot::Sender<()>,
) -> Write {
    tokio::spawn(async move {
        let mut write = Box::pin(store.write_range_for_transfer_if_current(&identity, 3, b"late"));
        poll_fn(|context| match write.as_mut().poll(context) {
            Poll::Pending => Poll::Ready(()),
            Poll::Ready(_) => panic!("waiting write unexpectedly completed"),
        })
        .await;
        let _ = ready.send(());
        write.await
    })
}

async fn assert_rejected(persisting: Write, waiting: Write) {
    assert!(!persisting.await.unwrap().unwrap());
    assert!(!waiting.await.unwrap().unwrap());
}

async fn assert_discarded(store: &PartialRangeStore, root: &Path, binding: &RepresentationBinding) {
    assert!(store.present_ranges("same").await.unwrap().is_empty());
    let stored = tokio::fs::read_to_string(root.join("same.representation"))
        .await
        .unwrap();
    assert_eq!(stored, binding.representation().fingerprint());
}

fn mirrored_meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://a.example/video".to_owned(),
            "https://b.example/video".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
