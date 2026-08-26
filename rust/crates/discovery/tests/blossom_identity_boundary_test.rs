use crate::content::candidates::CandidateRegistry;
use nostr_sdk::{EventBuilder, Keys, Kind, Tag};

#[test]
fn conflicting_x_or_ox_alone_never_creates_a_blossom_mirror_group() {
    let author = Keys::generate();
    let url_hash = "a".repeat(64);
    let other_hash = "b".repeat(64);
    let server = EventBuilder::new(Kind::Custom(10063), "")
        .tags([Tag::parse(["server", "https://blossom.example"]).expect("server")])
        .sign_with_keys(&author)
        .expect("server list");
    let conflict = video(
        &author,
        &format!("https://origin.example/{url_hash}.mp4"),
        &format!("x {other_hash}"),
    );
    let lineage = video(
        &author,
        "https://origin.example/plain.mp4",
        &format!("ox {url_hash}"),
    );

    let conflict = candidate(&[conflict, server.clone()]);
    assert_eq!(conflict.meta.sha256.as_deref(), Some(other_hash.as_str()));
    assert_eq!(conflict.meta.urls.len(), 1);
    let lineage = candidate(&[lineage, server]);
    assert_eq!(lineage.meta.sha256, None);
    assert_eq!(lineage.meta.urls.len(), 1);
}

fn video(author: &Keys, url: &str, identity: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), "clip")
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            "m video/mp4".to_owned(),
            identity.to_owned(),
        ])
        .expect("imeta")])
        .sign_with_keys(author)
        .expect("video")
}

fn candidate(events: &[nostr_sdk::Event]) -> crate::content::parsing::ParsedVideoPost {
    CandidateRegistry::new()
        .inspect_all(events)
        .admitted
        .into_iter()
        .next()
        .expect("candidate")
        .post
}
