use ghostr_hls_manifest::hls_manifest::rewrite_hls_manifest;
use url::Url;

#[test]
fn rejects_content_steering_that_could_escape_the_gateway() {
    let base = Url::parse("https://media.example/master.m3u8").expect("base URL");
    let playlist = b"#EXTM3U\n\
#EXT-X-CONTENT-STEERING:SERVER-URI=\"steering.json\",PATHWAY-ID=\"cdn-a\"\n\
#EXT-X-STREAM-INF:BANDWIDTH=800000\nvariant.m3u8\n";

    let error = rewrite_hls_manifest(playlist, &base, |_| {
        Ok("http://127.0.0.1/resource".to_owned())
    })
    .expect_err("content steering must fail closed");

    assert!(error.to_string().contains("CONTENT-STEERING"));
}
