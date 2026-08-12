use ghostr_hls_manifest::hls_manifest::{rewrite_hls_manifest, HlsResourceKind};
use url::Url;

#[test]
fn rewrites_encryption_keys_and_initialization_maps_as_assets() {
    let base = Url::parse("https://media.example/live/index.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n\
#EXT-X-KEY:METHOD=AES-128,URI=\"keys/key.bin?version=1\"\n\
#EXT-X-MAP:URI=\"https://init.example/video.mp4\",BYTERANGE=\"720@0\"\n\
#EXTINF:6,\nsegment.ts\n";
    let mut captured = Vec::new();

    let rewritten = rewrite_hls_manifest(playlist, &base, |resource| {
        let index = captured.len();
        captured.push(resource);
        Ok(format!("http://127.0.0.1:3000/hls/session/assets/{index}"))
    })
    .expect("rewrite playlist");

    assert!(rewritten.contains("URI=\"http://127.0.0.1:3000/hls/session/assets/0\""));
    assert!(rewritten.contains("URI=\"http://127.0.0.1:3000/hls/session/assets/1\""));
    assert_eq!(captured.len(), 3);
    assert!(captured
        .iter()
        .all(|resource| resource.kind == HlsResourceKind::Asset));
    assert_eq!(
        captured[0].url.as_str(),
        "https://media.example/live/keys/key.bin?version=1"
    );
    assert_eq!(captured[1].url.as_str(), "https://init.example/video.mp4");
}
