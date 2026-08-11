use ghostr_hls_manifest::hls_manifest::{rewrite_hls_manifest, HlsResourceKind};
use url::Url;

#[test]
fn rewrites_every_nested_playlist_reference() {
    let base = Url::parse("https://media.example/master.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n\
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"audio\",URI=\"audio/main.m3u8\"\n\
#EXT-X-STREAM-INF:BANDWIDTH=800000,AUDIO=\"audio\"\n\
https://variants.example/low.m3u8\n";
    let mut captured = Vec::new();

    let rewritten = rewrite_hls_manifest(playlist, &base, |resource| {
        let index = captured.len();
        captured.push(resource);
        Ok(format!(
            "http://127.0.0.1:3000/hls/session/manifests/{index}"
        ))
    })
    .expect("rewrite playlist");

    assert!(rewritten.contains("URI=\"http://127.0.0.1:3000/hls/session/manifests/0\""));
    assert!(rewritten.ends_with("http://127.0.0.1:3000/hls/session/manifests/1\n"));
    assert_eq!(captured.len(), 2);
    assert!(captured
        .iter()
        .all(|resource| resource.kind == HlsResourceKind::Manifest));
    assert_eq!(
        captured[0].url.as_str(),
        "https://media.example/audio/main.m3u8"
    );
    assert_eq!(
        captured[1].url.as_str(),
        "https://variants.example/low.m3u8"
    );
}
