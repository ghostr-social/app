use super::{
    DeliveryNetworkStatus, DeliveryPlayback, FocusGeneration, TransportRescue,
    TransportRescueFeedback,
};
use ghostr_engine::evidence::NostrMetadataEvidence;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, PostId, PreviewDescriptor, VideoMeta};

/// One post of the viewer's focus window with its discovery metadata.
#[derive(Clone, Debug)]
pub struct FocusItem {
    pub post: PostId,
    pub meta: VideoMeta,
}

/// A validated discovery candidate available to probes before focus ranks it.
#[derive(Clone, Debug)]
pub struct DeliveryCandidate {
    pub post: PostId,
    pub meta: VideoMeta,
    pub preview: Option<PreviewDescriptor>,
    pub metadata_evidence: Vec<NostrMetadataEvidence>,
    pub renditions: Vec<VideoRendition>,
    pub discovered_at: u64,
}

/// Validated inline preview evidence associated with one focused post.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusPreview {
    pub post: PostId,
    pub descriptor: PreviewDescriptor,
}

/// Whether a focus movement represents user navigation or system control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusTransition {
    UserNavigation,
    RosterChange,
    TransportRescue,
}

/// A full replacement of the focus window (plan §2 `ffi_update_focus`).
#[derive(Clone, Debug)]
pub struct DeliveryFocus {
    pub items: Vec<FocusItem>,
    pub previews: Vec<FocusPreview>,
    pub current_index: usize,
    pub watch_ms: u64,
    pub generation: FocusGeneration,
    pub transition: FocusTransition,
    pub rescue: Option<TransportRescue>,
}

impl DeliveryFocus {
    pub fn compatibility(items: Vec<FocusItem>, current_index: usize, watch_ms: u64) -> Self {
        Self {
            items,
            previews: Vec::new(),
            current_index,
            watch_ms,
            generation: FocusGeneration::compatibility(),
            transition: FocusTransition::UserNavigation,
            rescue: None,
        }
    }

    pub(crate) fn current_post(&self) -> Option<&PostId> {
        self.items
            .get(self.current_index.min(self.items.len().checked_sub(1)?))
            .map(|item| &item.post)
    }
}

/// Control events the manager reacts to.
#[derive(Debug)]
pub enum DeliveryCommand {
    Candidate(DeliveryCandidate),
    Focus(DeliveryFocus),
    #[doc(hidden)]
    RescueFeedback(TransportRescueFeedback),
    Playback(DeliveryPlayback),
    Config(DataUsageLevel),
    NetworkStatus(DeliveryNetworkStatus),
    NetworkProfile {
        generation: u64,
        profile: crate::debug::network::NetworkProfile,
    },
    StorageChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateAdmission {
    Accepted,
    Saturated,
    Closed,
}
