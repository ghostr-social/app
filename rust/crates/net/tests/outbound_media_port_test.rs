use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn default_media_transport_rejects_unexpected_ports() {
    let client = MediaHttpClient::public().expect("fixture");
    for url in [
        "https://1.1.1.1:22/video",
        "https://1.1.1.1:8080/video",
        "http://1.1.1.1:443/video",
    ] {
        assert!(
            client.get(url).is_err(),
            "unexpected media transport: {url}"
        );
    }
    assert!(client.get("https://1.1.1.1:443/video").is_ok());
    assert!(client.get("http://1.1.1.1:80/video").is_ok());
}
