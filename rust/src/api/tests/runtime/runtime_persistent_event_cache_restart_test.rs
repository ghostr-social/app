use crate::api::runtime::discovery::{DiscoveryBoot, DiscoveryRuntime};
use crate::discovery::cache::EventCache;
use crate::engine::adaptive::DiscoveryDemand;
use nostr_sdk::{Client, EventBuilder, Filter, Keys, Kind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn cold_runtime_restores_the_same_viewers_verified_events() {
    let root = cache_root();
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::TextNote, "offline")
        .sign_with_keys(&keys)
        .expect("signed event");
    let first = runtime(&root).await;
    first.reset_session(Some(keys.public_key())).await;
    first
        .remember_accepted(first.session_generation(), &event)
        .await;
    drop(first);

    let restored = runtime(&root).await;
    restored.reset_session(Some(keys.public_key())).await;
    let events = restored
        .executor
        .cache()
        .stored_for(restored.session_generation(), &Filter::new())
        .await
        .expect("current session");

    assert_eq!(
        events.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![event.id]
    );
    std::fs::remove_dir_all(root).expect("remove fixture");
}

async fn runtime(root: &Path) -> DiscoveryRuntime {
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Hold);
    let boot = DiscoveryBoot {
        client: Arc::new(Client::default()),
        demand,
        bootstrap: Vec::new(),
        search_relays: Vec::new(),
        candidates: None,
    };
    let cache = Arc::new(EventCache::persistent(root));
    DiscoveryRuntime::start_with_cache(boot, cache).await
}

fn cache_root() -> PathBuf {
    let name = format!("ghostr-runtime-event-cache-{}", std::process::id());
    std::env::temp_dir().join(name)
}
