use crate::api::focus_control::progressive_url;

#[test]
fn builds_the_loopback_progressive_url() {
    let url = progressive_url("127.0.0.1:8080", "clip-1", "opaque-token");

    assert_eq!(
        url,
        "http://127.0.0.1:8080/video.mp4?id=clip-1&cap=opaque-token"
    );
}
