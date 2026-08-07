use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn rejects_non_http_media_schemes() {
    let client = MediaHttpClient::public().expect("media client");

    let error = client
        .get("ftp://media.example/video.mp4")
        .expect_err("non-HTTP media URL");

    assert_eq!(error.to_string(), "media URL scheme is not allowed");
}
