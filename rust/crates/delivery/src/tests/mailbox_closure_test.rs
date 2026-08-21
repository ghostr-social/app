use crate::delivery_events::{
    command_channel, CandidateAdmission, DeliveryCandidate, DeliveryFocus, FocusAdmission,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn closed_mailbox_reports_rejection_and_ends_receiving() {
    let (handle, receiver) = command_channel();
    drop(receiver);

    handle.set_data_usage(DataUsageLevel::Balanced);
    assert_eq!(
        handle.update_focus(DeliveryFocus::compatibility(Vec::new(), 0, 0)),
        FocusAdmission::Closed
    );
    assert_eq!(
        handle.admit_candidate(candidate()),
        CandidateAdmission::Closed
    );

    let (handle, mut receiver) = command_channel();
    drop(handle);
    let (commands, _) = receiver.receivers();
    assert!(!commands.changed().await);
}

fn candidate() -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new("candidate"),
        meta: VideoMeta {
            urls: vec!["https://media.example/video.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at: 1,
    }
}
