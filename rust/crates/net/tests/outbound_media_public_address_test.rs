use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn accepts_literal_public_destinations() {
    let client = MediaHttpClient::public().expect("media client");

    assert!(client.get("https://1.1.1.1/video.mp4").is_ok());
    assert!(client
        .get("https://[2606:4700:4700::1111]/video.mp4")
        .is_ok());
}
