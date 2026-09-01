use super::{Fixture, NEW_BODY, OLD_PREFIX, SOURCE, STRONG_ETAG};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{HttpGenerationAuthority, SourceGeneration};
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

pub(in super::super) async fn seeded_fixture() -> Fixture {
    let root = crate::tests::store_fixture::temp_root("staged-stale-validator-revision");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), metadata());
    let transfer = binding.transfer(SOURCE).expect("transfer");
    store.bind_representation(binding).await.expect("binding");
    let authority = crate::tests::store_fixture::http_generation(SOURCE, "stable", 1);
    let HttpGenerationAuthority::Trusted(lease) = authority.clone() else {
        unreachable!()
    };
    store
        .apply_http_generation(&transfer, authority)
        .await
        .expect("HTTP generation");
    seed_sparse_prefix(&store, &transfer).await;
    Fixture {
        root,
        store,
        transfer,
        lease,
    }
}

async fn seed_sparse_prefix(
    store: &crate::partial_range_store::PartialRangeStore,
    transfer: &ghostr_engine::representation::TransferIdentity,
) {
    let generation = SourceGeneration::try_new(SOURCE, STRONG_ETAG, NEW_BODY.len() as u64)
        .expect("source generation");
    let action = store
        .reserve_action(transfer, 1, OLD_PREFIX.len() as u64)
        .await
        .expect("sparse reservation");
    store
        .open_sparse_response(transfer, &action, generation.clone(), ByteRange::new(0, 4))
        .await
        .expect("sparse open");
    store
        .write_range_for_action_if_current(transfer, &generation, &action, 0, OLD_PREFIX)
        .await
        .expect("sparse write");
    assert!(store
        .finish_sparse_response(transfer, &generation, &action)
        .await
        .expect("sparse finish"));
    store.release_action(&action).await;
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(NEW_BODY.len() as u64),
        duration_ms: Some(1_000),
    }
}
