use reqwest::Url;
use rust_lib_ghostr::video::hls_manifest::rewrite_hls_manifest;

#[test]
fn rejects_oversized_input_before_issuing_resources() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let mut manifest = b"#EXTM3U\n".to_vec();
    manifest.resize(1024 * 1024 + 1, b'a');
    let mut issued = 0;

    let result = rewrite_hls_manifest(&manifest, &base, |_| {
        issued += 1;
        Ok("/resource".to_owned())
    });

    assert!(result.is_err());
    assert_eq!(issued, 0);
}

#[test]
fn rejects_rewrites_whose_gateway_urls_exceed_the_output_bound() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let mut manifest = String::from("#EXTM3U\n");
    for _ in 0..200 {
        manifest.push_str("a\n");
    }

    let result = rewrite_hls_manifest(manifest.as_bytes(), &base, |_| Ok("x".repeat(100_000)));

    assert!(result.is_err());
}
