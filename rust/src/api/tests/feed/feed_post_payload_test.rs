//! `feed_post`: one parsed post plus its creator profile becomes the
//! full FFI row payload, with a gateway-safe post id that is stable
//! across addressable revisions of the same video.

use crate::api::delivery::focus_mapping::validate_post_id;
use crate::api::delivery_types::FfiMediaDelivery;
use crate::api::feed::mapping::{feed_post, post_gateway_id};
use crate::api::tests::support::{creator_profile, parsed_video_post};
use crate::discovery::content::parsing::ParsedVideoPost;
use crate::engine::DeliveryKind;

fn post(delivery: DeliveryKind) -> ParsedVideoPost {
    let mut post = parsed_video_post(34_235, Some("clip"));
    post.meta.delivery = delivery;
    post
}

#[test]
fn every_row_field_is_carried_over() {
    let mut post = post(DeliveryKind::Progressive);
    post.title = Some("Night skate".to_owned());
    let row = feed_post(&post, creator_profile(), None);
    assert_eq!(row.event_id, "e1");
    assert_eq!(row.created_at, 77);
    assert_eq!(row.caption, "sunset ride");
    assert_eq!(row.title.as_deref(), Some("Night skate"));
    assert_eq!(row.hashtags, vec!["sunset".to_owned()]);
    assert_eq!(
        (row.creator.pubkey, row.creator.display_name),
        ("a1".to_owned(), "Vera".to_owned())
    );
    assert_eq!(row.creator.handle, "@npub1vera");
    assert_eq!(
        row.creator.avatar_url.as_deref(),
        Some("https://cdn.example/a.png")
    );
    assert_eq!(row.media.urls, vec!["https://cdn.example/v.mp4".to_owned()]);
    assert_eq!(row.media.delivery, FfiMediaDelivery::Progressive);
    assert_eq!(row.media.sha256.as_deref(), Some("ff".repeat(32).as_str()));
    assert_eq!(
        (row.media.size_bytes, row.media.duration_ms),
        (Some(9), Some(2_000))
    );
    let dim = row.media.dim.expect("dimensions carried");
    assert_eq!((dim.width, dim.height), (608, 1080));
    assert_eq!(row.media.blurhash.as_deref(), Some("LKO2"));
    assert_eq!(
        row.media.thumb_url.as_deref(),
        Some("https://cdn.example/t.jpg")
    );
}

#[test]
fn hls_delivery_round_trips_the_v1_focus_vocabulary() {
    let row = feed_post(&post(DeliveryKind::Hls), creator_profile(), None);
    assert_eq!(row.media.delivery, FfiMediaDelivery::Hls);
}

#[test]
fn post_ids_are_gateway_safe_and_stable_per_coordinate() {
    let row = feed_post(&post(DeliveryKind::Progressive), creator_profile(), None);
    validate_post_id(&row.post_id).expect("gateway-safe id");
    let mut revision = post(DeliveryKind::Progressive);
    revision.event_id = "e2".to_owned();
    revision.created_at = 90;
    assert_eq!(post_gateway_id(&revision), row.post_id);
    let mut other = post(DeliveryKind::Progressive);
    other.identifier = Some("other-clip".to_owned());
    other.published_identifier = Some("other-clip".to_owned());
    assert_ne!(post_gateway_id(&other), row.post_id);
}
