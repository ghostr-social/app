use ghostr_discovery::content::parsing::video_post_from_event;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};

fn signed(content: &str, tags: Vec<Tag>) -> Event {
    EventBuilder::new(Kind::Custom(22), content)
        .tags(tags)
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}

fn imeta(fields: &[&str]) -> Tag {
    let mut values = vec!["imeta"];
    values.extend(fields);
    Tag::parse(values).expect("imeta tag")
}

fn bad_imeta() -> Tag {
    // A present but invalid digest rejects the whole imeta tag.
    imeta(&[
        "url https://imeta.example/bad.mp4",
        "m video/mp4",
        "x nothex",
    ])
}

fn file_tags() -> Vec<Tag> {
    vec![
        Tag::parse(["url", "https://file.example/f.mp4"]).expect("url tag"),
        Tag::parse(["m", "video/mp4"]).expect("m tag"),
    ]
}

#[test]
fn event_parsing_prefers_imeta_over_file_tags_and_text() {
    // Resolution order is imeta, NIP-94 file tags, then note text.
    let mut tags = file_tags();
    tags.push(imeta(&["url https://imeta.example/i.mp4", "m video/mp4"]));
    let event = signed("text https://text.example/t.mp4", tags);
    let post = video_post_from_event(&event).expect("parsed post");
    assert_eq!(post.meta.urls, ["https://imeta.example/i.mp4"]);
}

#[test]
fn event_parsing_skips_an_invalid_imeta_for_the_next_valid_one() {
    let event = signed(
        "clip",
        vec![
            bad_imeta(),
            imeta(&["url https://imeta.example/good.mp4", "m video/mp4"]),
        ],
    );
    let post = video_post_from_event(&event).expect("parsed post");
    assert_eq!(post.meta.urls, ["https://imeta.example/good.mp4"]);
}

#[test]
fn event_parsing_falls_back_to_file_tags_then_text() {
    let mut with_file_tags = vec![bad_imeta()];
    with_file_tags.extend(file_tags());
    let file_post = video_post_from_event(&signed("clip", with_file_tags)).expect("file post");
    assert_eq!(file_post.meta.urls, ["https://file.example/f.mp4"]);

    let text_post = video_post_from_event(&signed(
        "clip https://text.example/t.mp4",
        vec![bad_imeta()],
    ))
    .expect("text post");
    assert_eq!(text_post.meta.urls, ["https://text.example/t.mp4"]);

    let nothing = video_post_from_event(&signed("clip", vec![bad_imeta()]));
    assert!(nothing.is_none(), "no media source is left to fall back to");
}
