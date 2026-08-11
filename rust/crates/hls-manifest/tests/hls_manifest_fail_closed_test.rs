use ghostr_hls_manifest::hls_manifest::{rewrite_hls_manifest, HlsResourceKind};
use url::Url;

#[test]
fn rewrites_low_latency_and_image_playlist_resources() {
    let base = Url::parse("https://media.example/live/master.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n\
#EXT-X-PART:DURATION=0.333,URI=\"parts/1.m4s\"\n\
#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"parts/2.m4s\"\n\
#EXT-X-RENDITION-REPORT:URI=\"alt.m3u8\",LAST-MSN=12\n\
#EXT-X-IMAGE-STREAM-INF:BANDWIDTH=1200,URI=\"images.m3u8\"\n";
    let mut resources = Vec::new();

    let rewritten = rewrite_hls_manifest(playlist, &base, |resource| {
        let kind = resource.kind;
        resources.push(resource);
        Ok(format!("/secure/{kind:?}"))
    })
    .expect("valid manifest");

    assert!(!rewritten.contains("parts/"));
    assert!(!rewritten.contains("alt.m3u8"));
    assert_eq!(resources[0].kind, HlsResourceKind::Asset);
    assert_eq!(resources[2].kind, HlsResourceKind::Manifest);
    assert_eq!(resources[3].kind, HlsResourceKind::Manifest);
}

#[test]
fn rejects_variable_substitution_and_interstitial_urls() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    for playlist in [
        "#EXTM3U\n#EXT-X-DEFINE:NAME=\"host\",VALUE=\"cdn.example\"\n",
        "#EXTM3U\n#EXT-X-DATERANGE:ID=\"ad\",X-ASSET-URI=\"ad.m3u8\"\n",
    ] {
        assert!(rewrite_hls_manifest(playlist.as_bytes(), &base, |_| unreachable!()).is_err());
    }
}

#[test]
fn rejects_unknown_extensions_and_dangling_variant_uris() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    for playlist in [
        "#EXTM3U\n#EXT-X-NEW-REMOTE:URI=\"escape.example\"\n",
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000\n",
    ] {
        assert!(rewrite_hls_manifest(playlist.as_bytes(), &base, |_| unreachable!()).is_err());
    }
}

#[test]
fn preserves_safe_metadata_empty_lines_and_line_endings() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\r\n\r\n#EXT-X-DATERANGE\r\n\
#EXT-X-MEDIA:TYPE=CLOSED-CAPTIONS,GROUP-ID=\"cc\"\r\n#EXT-X-ENDLIST";

    let rewritten = rewrite_hls_manifest(playlist, &base, |_| unreachable!())
        .expect("safe metadata-only manifest");

    assert_eq!(rewritten.as_bytes(), playlist);
}

#[test]
fn rejects_malformed_or_credentialed_resource_declarations() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    for playlist in [
        "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=key.bin\n",
        "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\n",
        "#EXTM3U\n#EXT-X-KEY:METHOD\n",
        "#EXTM3U\n#EXT-X-KEY:method=AES-128\n",
        "#EXTM3U\n#EXT-X-MAP:BYTERANGE=\"100@0\"\n",
        "#EXTM3U\nhttps://user:secret@media.example/segment.ts\n",
        "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\n#EXT-X-STREAM-INF:BANDWIDTH=2\n",
    ] {
        assert!(rewrite_hls_manifest(playlist.as_bytes(), &base, |_| {
            Ok("/secure/resource".to_owned())
        })
        .is_err());
    }
}

#[test]
fn rejects_duplicate_or_malformed_trailing_uri_attributes() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    for playlist in [
        "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"safe.key\",URI=\"http://127.0.0.1/private\"\n",
        "#EXTM3U\n#EXT-X-KEY:URI=\"safe.key\",METHOD=AES-128,URI=unsafe.key\n",
    ] {
        let mut issued = 0;
        let result = rewrite_hls_manifest(playlist.as_bytes(), &base, |_| {
            issued += 1;
            Ok("/secure/key".to_owned())
        });
        assert!(result.is_err());
        assert_eq!(issued, 0, "rewrites must be transactional");
    }
}
