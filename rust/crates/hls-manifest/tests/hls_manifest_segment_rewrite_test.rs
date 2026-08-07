use url::Url;
use ghostr_hls_manifest::hls_manifest::{rewrite_hls_manifest, HlsResourceKind};

#[test]
fn rewrites_relative_media_segments_to_gateway_resources() {
    let base = Url::parse("https://media.example/live/index.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXTINF:6,\nsegments/one.ts\n";
    let mut captured = Vec::new();

    let rewritten = rewrite_hls_manifest(playlist, &base, |resource| {
        captured.push(resource);
        Ok("http://127.0.0.1:3000/hls/session/assets/0".to_owned())
    })
    .expect("rewrite playlist");

    assert_eq!(
        rewritten,
        "#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXTINF:6,\n\
         http://127.0.0.1:3000/hls/session/assets/0\n"
    );
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].kind, HlsResourceKind::Asset);
    assert_eq!(
        captured[0].url.as_str(),
        "https://media.example/live/segments/one.ts"
    );
}
