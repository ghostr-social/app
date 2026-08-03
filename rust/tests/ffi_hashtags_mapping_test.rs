mod support;

use rust_lib_ghostr::video::ffi_models::ffi_video_download;
use support::fixtures::native_download;

#[test]
fn carries_event_hashtags_across_the_ffi_mapping() {
    let mut native = native_download("https://media.example/video.mp4");
    native.event.hashtags = vec!["dance".to_owned(), "footwork".to_owned()];

    let mapped = ffi_video_download(&native);

    assert_eq!(mapped.event.hashtags, vec!["dance", "footwork"]);
}
