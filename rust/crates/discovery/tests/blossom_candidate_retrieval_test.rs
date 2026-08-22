use ghostr_discovery::content::candidates::{CandidateAdmission, CandidateRegistry};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Tag, Timestamp};

#[test]
fn author_server_list_enriches_a_progress_candidate_in_authored_order() {
    let author = Keys::generate();
    let digest = "A1".repeat(32);
    let video = video(&author, &digest);
    let mut registry = CandidateRegistry::new();
    assert!(matches!(
        registry.inspect(&video).admission,
        CandidateAdmission::Accepted(_)
    ));
    let old = servers(&author, 10, &["https://old.example"]);
    let current = servers(
        &author,
        20,
        &[
            "https://first.example/base/",
            "https://second.example",
            "https://first.example/base",
            "https://user@rejected.example",
        ],
    );
    let outsider = servers(&Keys::generate(), 30, &["https://outsider.example"]);

    let batch = registry.inspect_all(&[video.clone(), old, current, outsider]);
    let candidate = batch
        .admitted
        .into_iter()
        .next()
        .expect("enriched candidate");
    let hash = digest.to_ascii_lowercase();
    assert_eq!(candidate.post.meta.sha256.as_deref(), Some(hash.as_str()));
    assert_eq!(
        candidate.post.meta.urls,
        [
            format!("https://origin.example/{digest}.mp4"),
            format!("https://first.example/base/{hash}"),
            format!("https://second.example/{hash}"),
        ]
    );
}

fn video(author: &Keys, digest: &str) -> Event {
    EventBuilder::new(Kind::Custom(22), "clip")
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url https://origin.example/{digest}.mp4"),
            "m video/mp4".to_owned(),
        ])
        .expect("imeta")])
        .sign_with_keys(author)
        .expect("video")
}

fn servers(author: &Keys, created_at: u64, urls: &[&str]) -> Event {
    let tags = urls
        .iter()
        .map(|url| Tag::parse(["server", url]).expect("server"));
    EventBuilder::new(Kind::Custom(10063), "")
        .tags(tags)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(author)
        .expect("server list")
}
