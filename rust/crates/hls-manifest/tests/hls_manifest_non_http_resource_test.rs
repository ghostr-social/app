use ghostr_hls_manifest::hls_manifest::rewrite_hls_manifest;
use url::Url;

#[test]
fn rejects_non_http_resources_before_issuing_a_gateway_route() {
    let base = Url::parse("https://media.example/live.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXTINF:6,\nfile:///etc/passwd\n";
    let mut issued = false;

    let error = rewrite_hls_manifest(playlist, &base, |_| {
        issued = true;
        Ok("http://127.0.0.1/resource".to_owned())
    })
    .expect_err("non-HTTP resource must fail closed");

    assert!(error.to_string().contains("scheme"));
    assert!(!issued);
}
