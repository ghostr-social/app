use super::snapshots::DeliverySnapshot;
use ghostr_delivery::segmented::{SegmentedPhase, SegmentedSnapshot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeliverySnapshotHlsAuthority {
    pub delivery_id: String,
    pub representation_id: String,
    pub asset_revision: u64,
}

pub(crate) fn hls_snapshot(snapshot: SegmentedSnapshot) -> DeliverySnapshot {
    let hls_authority = snapshot
        .authority
        .map(|authority| DeliverySnapshotHlsAuthority {
            delivery_id: authority.post().as_str().to_owned(),
            representation_id: authority.representation_id().fingerprint().to_owned(),
            asset_revision: authority.asset_revision().value(),
        });
    DeliverySnapshot {
        startable: snapshot.phase == SegmentedPhase::Ready,
        bytes_present: snapshot.bytes_present,
        total_bytes: None,
        eta_ms: snapshot.eta_ms,
        failed: snapshot.phase == SegmentedPhase::Failed,
        detail: snapshot.detail,
        authority: None,
        hls_authority,
    }
}
