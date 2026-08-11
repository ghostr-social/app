use crate::manager::plan::PlannedTransfer;
use crate::mutable_priority_queue::MutablePriorityQueue;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::{ByteRange, ChunkId, DeliveryKind, PostId, VideoMeta};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
    std::fs::create_dir_all(&path).expect("create test directory");
    path
}

pub(crate) fn transfer_identity(post: &PostId, url: &str) -> TransferIdentity {
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![url.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1),
            duration_ms: Some(1),
        },
    );
    catalog.transfer_identity(post, url).expect("test source")
}

pub(crate) fn chunk_request(chunk: ChunkId, tier: Tier) -> ChunkRequest {
    ChunkRequest {
        chunk,
        tier,
        score: 1.0,
        startup_depth_bytes: 0,
    }
}

pub(crate) fn planned_transfer(name: &str, host: &str, tier: Tier) -> PlannedTransfer {
    let post = PostId::new(name);
    let url = format!("https://{host}/{name}.mp4");
    PlannedTransfer {
        identity: transfer_identity(&post, &url),
        request: ChunkRequest {
            chunk: ChunkId {
                post,
                range: ByteRange::new(0, 4),
            },
            tier,
            score: 1.0,
            startup_depth_bytes: 0,
        },
        url,
    }
}

pub(crate) fn planned_queue(items: &[(&str, Tier)], host: &str) -> MutablePriorityQueue {
    let mut queue = MutablePriorityQueue::new();
    queue.replace(
        items
            .iter()
            .map(|(name, tier)| planned_transfer(name, host, *tier))
            .collect(),
    );
    queue
}

pub(crate) fn active_hosts(host: &str) -> HashSet<String> {
    HashSet::from([host.to_owned()])
}

pub(crate) fn transfer_posts<const N: usize>(items: &[PlannedTransfer; N]) -> [String; N] {
    std::array::from_fn(|index| items[index].request.chunk.post.as_str().to_owned())
}
