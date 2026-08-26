use ghostr_hls_manifest::hls_manifest::{inspect_hls_bootstrap, HlsBootstrap};
use url::Url;

#[test]
fn master_selects_one_bounded_variant() {
    let body = b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000\nlow/index.m3u8\n\
                 #EXT-X-STREAM-INF:BANDWIDTH=1600000\nhigh/index.m3u8\n";
    let base = Url::parse("https://cdn.example/root/master.m3u8").expect("valid test fixture");

    let inspected = inspect_hls_bootstrap(body, &base).expect("valid test fixture");

    assert_eq!(
        inspected,
        HlsBootstrap::Master {
            variant: Url::parse("https://cdn.example/root/low/index.m3u8")
                .expect("valid test fixture"),
        }
    );
}

#[test]
fn vod_media_identifies_init_and_first_playable_segment() {
    let body = b"#EXTM3U\n#EXT-X-TARGETDURATION:4\n\
                 #EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment-1.m4s\n#EXT-X-ENDLIST\n";
    let base = Url::parse("https://cdn.example/video/index.m3u8").expect("valid test fixture");

    let inspected = inspect_hls_bootstrap(body, &base).expect("valid test fixture");

    assert_eq!(
        inspected,
        HlsBootstrap::Media {
            init: Some(
                Url::parse("https://cdn.example/video/init.mp4").expect("valid test fixture")
            ),
            segment: Url::parse("https://cdn.example/video/segment-1.m4s")
                .expect("valid test fixture"),
        }
    );
}
