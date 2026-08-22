use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn rejects_credentials_in_initial_media_urls() {
    let client = MediaHttpClient::public().expect("media client");

    for url in [
        "https://user@example.com/video.mp4",
        "https://user:password@example.com/video.mp4",
    ] {
        let error = client.get(url).expect_err("credential-bearing media URL");
        assert_eq!(error.to_string(), "media URL credentials are forbidden");
    }
}
