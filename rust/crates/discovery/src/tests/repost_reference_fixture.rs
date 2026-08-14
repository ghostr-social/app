use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag};

pub(crate) fn signed_wrapper(
    keys: &Keys,
    kind: u16,
    content: impl Into<String>,
    tags: Vec<Tag>,
) -> Event {
    EventBuilder::new(Kind::Custom(kind), content)
        .tags(tags)
        .sign_with_keys(keys)
        .expect("wrapper")
}

pub(crate) fn video(keys: &Keys, kind: Kind, tags: Vec<Tag>) -> Event {
    EventBuilder::new(kind, "https://cdn.example/video.mp4")
        .tags(tags)
        .sign_with_keys(keys)
        .expect("video")
}

pub(crate) fn tag(values: &[&str]) -> Tag {
    Tag::parse(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("tag")
}
