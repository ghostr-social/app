use rust_lib_ghostr::video::video_link_scan::first_video_link;

#[test]
fn video_link_scan_skips_http_text_that_is_not_a_link_scheme() {
    let content = "bad httpx://media.example/ignored.mp4 then https://media.example/accepted.webm";

    assert_eq!(
        first_video_link(content).as_deref(),
        Some("https://media.example/accepted.webm")
    );
}
