use nostr_sdk::{Event, EventBuilder, Keys, Kind};
use ghostr_discovery::event_parsing::video_post_from_event;
use ghostr_engine::DeliveryKind;

fn note(content: &str) -> Event {
    EventBuilder::new(Kind::Custom(1), content)
        .sign_with_keys(&Keys::generate())
        .expect("signed note")
}

#[test]
fn event_parsing_scrapes_the_first_video_link_from_a_kind1_note() {
    // The leftmost recognized extension wins after trailing punctuation
    // is trimmed.
    let cases = [
        (
            "watch https://cdn.example/clip.mp4!",
            "https://cdn.example/clip.mp4",
            DeliveryKind::Progressive,
        ),
        (
            "(https://cdn.example/live.m3u8)",
            "https://cdn.example/live.m3u8",
            DeliveryKind::Hls,
        ),
        (
            "https://a.example/page https://b.example/one.webm https://c.example/two.mp4",
            "https://b.example/one.webm",
            DeliveryKind::Progressive,
        ),
        (
            "caps https://cdn.example/CLIP.MOV",
            "https://cdn.example/CLIP.MOV",
            DeliveryKind::Progressive,
        ),
        (
            "trailing https://cdn.example/c.m4v,\"",
            "https://cdn.example/c.m4v",
            DeliveryKind::Progressive,
        ),
    ];
    for (content, url, delivery) in cases {
        let post = video_post_from_event(&note(content)).expect(content);
        assert_eq!(post.meta.urls, [url], "{content}");
        assert_eq!(post.meta.delivery, delivery, "{content}");
        assert_eq!(post.meta.sha256, None, "{content}");
        assert_eq!(post.meta.size_bytes, None, "{content}");
    }
}

#[test]
fn event_parsing_strips_the_scraped_link_from_the_caption() {
    let post = video_post_from_event(&note("new edit https://cdn.example/clip.mp4 enjoy"))
        .expect("parsed post");
    assert_eq!(post.caption, "new edit enjoy");
    assert_eq!(post.title, None);
}

#[test]
fn event_parsing_rejects_notes_without_a_direct_video_link() {
    let cases = [
        "no links at all",
        "https://cdn.example/photo.jpg",
        "https://cdn.example/clip.mp4.html",
        "ftp://cdn.example/clip.mp4",
        "extension must live on the path https://cdn.example/clip?format=mp4",
        "HTTPS://cdn.example/upper-scheme.mp4",
    ];
    for content in cases {
        assert!(video_post_from_event(&note(content)).is_none(), "{content}");
    }
}
