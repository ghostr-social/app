use super::super::test_fixture::TransformFixture;
use crate::transform::TransformControl;
use ghostr_partial_store::partial_range_store::TransformPublicationOutcome;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_wins_at_store_boundary_without_publishing_bytes() {
    let fixture = TransformFixture::seeded("transform-cancel-wins").await;
    let control = control();
    let (entered, release) = barriers();
    let publication = fixture.publication();
    let task = tokio::spawn({
        let store = fixture.store.clone();
        let control = control.clone();
        let entered = entered.clone();
        let release = release.clone();
        async move {
            store
                .publish_transform_authorized(publication, move || {
                    entered.wait();
                    release.wait();
                    control.try_begin_commit()
                })
                .await
        }
    });
    entered.wait();
    assert!(control.cancel());
    release.wait();
    assert_eq!(
        task.await.unwrap().unwrap(),
        TransformPublicationOutcome::Cancelled
    );
    assert_eq!(
        fixture.store.read_range("post", 0..4).await.unwrap(),
        Some(b"data".to_vec())
    );
    assert_eq!(fixture.store.used_bytes().await, 4);
    assert!(!fixture.has_transform_staging());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn commit_wins_at_store_boundary_and_rejects_late_cancel() {
    let fixture = TransformFixture::seeded("transform-commit-wins").await;
    let control = control();
    let (entered, release) = barriers();
    let publication = fixture.publication();
    let task = tokio::spawn({
        let store = fixture.store.clone();
        let control = control.clone();
        let entered = entered.clone();
        let release = release.clone();
        async move {
            store
                .publish_transform_authorized(publication, move || {
                    let accepted = control.try_begin_commit();
                    entered.wait();
                    release.wait();
                    accepted
                })
                .await
        }
    });
    entered.wait();
    assert!(!control.cancel());
    release.wait();
    assert_eq!(
        task.await.unwrap().unwrap(),
        TransformPublicationOutcome::Published
    );
    assert_eq!(
        fixture.store.read_range("post", 0..5).await.unwrap(),
        Some(b"done!".to_vec())
    );
    assert_eq!(fixture.store.used_bytes().await, 5);
}

fn control() -> TransformControl {
    TransformControl::new(Instant::now() + Duration::from_secs(1))
}

fn barriers() -> (Arc<Barrier>, Arc<Barrier>) {
    (Arc::new(Barrier::new(2)), Arc::new(Barrier::new(2)))
}
