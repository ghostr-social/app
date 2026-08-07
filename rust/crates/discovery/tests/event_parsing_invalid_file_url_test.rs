use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use ghostr_discovery::content::parsing::video_post_from_event;

#[test]
fn invalid_file_tag_url_falls_back_to_the_direct_video_link() {
    let event = EventBuilder::new(Kind::Custom(1063), "fallback https://text.example/clip.mp4")
        .tags([
            Tag::parse(["url", "ftp://file.example/clip.mp4"]).expect("URL tag"),
            Tag::parse(["m", "video/mp4"]).expect("mime tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");

    let post = video_post_from_event(&event).expect("text fallback");

    assert_eq!(post.meta.urls, ["https://text.example/clip.mp4"]);
}
