use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use ghostr_discovery::event_parsing::video_post_from_event;

const LINK: &str = "https://cdn.example/clip.mp4";

fn signed(kind: u16, content: &str, tags: Vec<Tag>) -> Event {
    EventBuilder::new(Kind::Custom(kind), content)
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

#[test]
fn event_parsing_accepts_the_video_discovery_query_kinds() {
    // The contract admits every NIP-71 kind, kind-1 notes carrying a
    // video link, and NIP-94 kind-1063 file events.
    let d_tag = Tag::parse(["d", "clip"]).expect("d tag");
    let accepted = [
        signed(1, &format!("note {LINK}"), vec![]),
        signed(21, &format!("normal {LINK}"), vec![]),
        signed(22, &format!("short {LINK}"), vec![]),
        signed(34235, &format!("addressable {LINK}"), vec![d_tag.clone()]),
        signed(34236, &format!("addressable {LINK}"), vec![d_tag]),
        signed(
            1063,
            "file event",
            vec![
                Tag::parse(["url", LINK]).expect("url tag"),
                Tag::parse(["m", "video/mp4"]).expect("m tag"),
            ],
        ),
    ];
    for event in accepted {
        assert!(
            video_post_from_event(&event).is_some(),
            "kind {}",
            event.kind.as_u16()
        );
    }
}

#[test]
fn event_parsing_rejects_kinds_outside_the_discovery_queries() {
    let rejected = [
        signed(0, &format!("profile {LINK}"), vec![]),
        signed(6, &format!("repost {LINK}"), vec![]),
        signed(
            30023,
            &format!("article {LINK}"),
            vec![Tag::parse(["d", "post"]).expect("d tag")],
        ),
        // Addressable video without its `d` identifier is invalid.
        signed(34235, &format!("addressable {LINK}"), vec![]),
        // A kind-1063 file event must declare an accepted video mime.
        signed(
            1063,
            &format!("file {LINK}"),
            vec![Tag::parse(["url", LINK]).expect("url tag")],
        ),
    ];
    for event in rejected {
        assert!(
            video_post_from_event(&event).is_none(),
            "kind {}",
            event.kind.as_u16()
        );
    }
}
