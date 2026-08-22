use crate::delivery_events::{
    command_channel_with_candidate_capacity, CandidateAdmission, DeliveryCandidate,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn candidate_ingress_reports_saturation_without_growing_its_backlog() {
    let (handle, mut receiver) = command_channel_with_candidate_capacity(2);

    assert_eq!(
        handle.admit_candidate(candidate("a")),
        CandidateAdmission::Accepted
    );
    assert_eq!(
        handle.admit_candidate(candidate("b")),
        CandidateAdmission::Accepted
    );
    assert_eq!(
        handle.admit_candidate(candidate("c")),
        CandidateAdmission::Saturated
    );

    assert_eq!(receiver.try_candidate().unwrap().post.as_str(), "a");
    assert_eq!(receiver.try_candidate().unwrap().post.as_str(), "b");
    assert!(receiver.try_candidate().is_none());
}

fn candidate(id: &str) -> DeliveryCandidate {
    DeliveryCandidate {
        post: PostId::new(id),
        meta: VideoMeta {
            urls: vec![format!("https://{id}.example/video.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: Vec::new(),
        discovered_at: 1,
    }
}
