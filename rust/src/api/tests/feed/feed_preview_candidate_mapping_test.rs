use crate::api::delivery::candidates::delivery_candidate;
use crate::discovery::content::candidates::{CandidateAdmission, CandidateRegistry};
use crate::engine::PreviewDescriptor;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

const BLURHASH: &str = "LEHV6nWB2yk8pyo0adR*.7kCMdnj";

#[test]
fn only_a_valid_inline_blurhash_becomes_a_delivery_preview() {
    let preview = mapped_candidate(Some(BLURHASH));
    assert_eq!(
        preview.preview,
        PreviewDescriptor::inline_blurhash(BLURHASH)
    );

    assert_eq!(mapped_candidate(Some("not!blurhash")).preview, None);
    assert_eq!(mapped_candidate(None).preview, None);
}

fn mapped_candidate(blurhash: Option<&str>) -> ghostr_delivery::delivery_events::DeliveryCandidate {
    let mut values = vec![
        "imeta".to_owned(),
        "url https://cdn.example/video.mp4".to_owned(),
        "m video/mp4".to_owned(),
        "image https://cdn.example/thumbnail.jpg".to_owned(),
    ];
    values.extend(blurhash.map(|value| format!("blurhash {value}")));
    let event = EventBuilder::new(Kind::Custom(22), "video")
        .tags([Tag::parse(values).expect("imeta")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let admission = CandidateRegistry::new().inspect(&event).admission;
    let CandidateAdmission::Accepted(candidate) = admission else {
        panic!("video candidate")
    };
    delivery_candidate(candidate)
}
