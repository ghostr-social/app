use crate::api::feed::mapping::feed_post;
use crate::api::feed_types::FfiFeedRepostTarget;
use crate::api::tests::support::{creator_profile, parsed_video_post};
use crate::discovery::content::profiles::CreatorProfile;
use crate::discovery::content::reposts::{RepostProvenance, RepostTarget};

#[test]
fn ffi_row_keeps_original_fields_and_adds_resolved_reposter() {
    let mut post = parsed_video_post(22, None);
    post.signed_event_json = Some(std::sync::Arc::from("{\"signed\":true}"));
    post.feed_sort_at = 99;
    post.repost = Some(RepostProvenance {
        event_id: "f1".to_owned(),
        reposter_pubkey: "b1".to_owned(),
        kind: 16,
        reposted_at: 99,
        target: RepostTarget::SpecificEvent,
    });
    let reposter = CreatorProfile {
        display_name: "Bob".to_owned(),
        handle: "@bob".to_owned(),
        avatar_url: None,
    };

    let row = feed_post(&post, creator_profile(), Some(reposter));

    assert_eq!(row.event_id, "e1");
    assert_eq!(row.created_at, 77);
    assert_eq!(row.feed_sort_at, 99);
    assert_eq!(row.signed_event_json.as_deref(), Some("{\"signed\":true}"));
    let repost = row.repost.expect("repost payload");
    assert_eq!(repost.event_id, "f1");
    assert_eq!(repost.event_kind, 16);
    assert_eq!(repost.target, FfiFeedRepostTarget::SpecificEvent);
    assert_eq!(repost.reposter.display_name, "Bob");
}
