use super::support::whole_profile;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoreAction};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
mod authorization;
mod failing_finish;
mod origin;
pub(super) use authorization::AuthorizedTraffic;
pub(super) use failing_finish::FailingFinishSink;
pub(super) use origin::split;
pub(super) struct SinkFixture {
    pub(super) root: PathBuf,
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) used: Arc<Mutex<u64>>,
    pub(super) identity: TransferIdentity,
    pub(super) action: StoreAction,
    pub(super) client: MediaRequestExecutor,
}

pub(super) struct LocalClient(reqwest::Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub(super) async fn fixture(prefix: &str, url: &str, seed: Option<&[u8]>) -> SinkFixture {
    let root = super::support::temp_directory(prefix);
    let used = Arc::new(Mutex::new(0));
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        std::sync::Arc::clone(&used),
        StoreCapacity::system(u64::MAX),
    ));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta(url));
    let identity = binding.transfer(url).expect("valid test fixture");
    store.bind_representation(binding).await.expect("valid test fixture");
    store.select_transfer(identity.clone()).await.expect("valid test fixture");
    if let Some(bytes) = seed {
        store.write_range("post", 0, bytes).await.expect("valid test fixture");
    }
    let action = store.reserve_action(&identity, 1, 64).await.expect("valid test fixture");
    SinkFixture {
        root,
        store,
        used,
        identity,
        action,
        client: MediaRequestExecutor::new(
            Arc::new(LocalClient(
                reqwest::Client::builder()
                    .no_proxy()
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .expect("valid test fixture"),
            )),
            MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
        ),
    }
}

pub(super) fn whole_spec<'a>(
    requests: &'a MediaRequestExecutor,
    url: &'a str,
    contract: WholeBodyContract,
) -> crate::chunk::downloader::ChunkSpec<'a> {
    crate::chunk::downloader::ChunkSpec {
        requests,
        url,
        request: RetrievalRequest::FetchWhole {
            contract,
            reason: WholeFetchReason::DirectCrossover,
        },
        attempt_profile: whole_profile(contract.maximum_bytes()),
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        continuation: None,
        timeouts: TransferTimeouts::default(),
    }
}

fn meta(url: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(1_000),
    }
}
