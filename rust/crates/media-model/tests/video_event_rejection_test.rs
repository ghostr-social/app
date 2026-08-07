use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};
use ghostr_media_model::event_identity::canonical_native_videos;

const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn rejects_events_that_are_not_canonical_nip71_videos() {
    let valid = imeta(&[
        "url https://media.example/video.mp4",
        "x aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "m video/mp4",
    ]);
    let cases = [
        event(1, vec![valid.clone()]),
        event(34235, vec![valid]),
        event(22, vec![Tag::parse(["alt", "clip"]).expect("alt tag")]),
        event(
            22,
            vec![imeta(&[
                "url https://media.example/video.mp4",
                &format!("x {HASH}"),
                "m image/png",
            ])],
        ),
        event(22, vec![imeta(&[&format!("x {HASH}"), "m video/mp4"])]),
        event(
            22,
            vec![imeta(&[
                "url ftp://media.example/video.mp4",
                &format!("x {HASH}"),
                "m video/mp4",
            ])],
        ),
    ];

    for candidate in cases {
        assert!(canonical_native_videos(&candidate).is_empty());
    }
}

fn event(kind: u16, tags: Vec<Tag>) -> Event {
    EventBuilder::new(Kind::Custom(kind), "clip")
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

fn imeta(fields: &[&str]) -> Tag {
    let mut values = vec!["imeta"];
    values.extend(fields);
    Tag::parse(values).expect("imeta tag")
}
