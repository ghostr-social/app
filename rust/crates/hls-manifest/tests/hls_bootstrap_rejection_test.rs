use ghostr_hls_manifest::hls_manifest::inspect_hls_bootstrap;
use url::Url;

#[test]
fn rejects_live_encrypted_and_low_latency_playlists() {
    let base = Url::parse("https://cdn.example/video/index.m3u8").expect("valid test fixture");
    let cases: &[&[u8]] = &[
        b"#EXTM3U\n#EXTINF:4,\nsegment.ts\n",
        b"#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key\"\n\
          #EXTINF:4,\nsegment.ts\n#EXT-X-ENDLIST\n",
        b"#EXTM3U\n#EXT-X-PART:DURATION=0.2,URI=\"part.m4s\"\n\
          #EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    ];

    for body in cases {
        assert!(inspect_hls_bootstrap(body, &base).is_err());
    }
}
