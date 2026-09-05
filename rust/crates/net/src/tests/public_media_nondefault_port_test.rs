use crate::outbound_media_client::MediaHttpClient;

#[test]
fn public_https_video_preserves_the_publishers_nondefault_port() {
    let client = MediaHttpClient::public().expect("public media client");
    let url = "https://mastodon-media.neometropolis.net:27892/media_attachments/files/117/218/447/478/165/335/original/6795387e110e5178.mp4";
    let request = client
        .get(url)
        .expect("public Nostr media URL must be accepted")
        .build()
        .expect("media request");
    assert_eq!(request.url().port(), Some(27892));
}

#[test]
fn nondefault_ports_never_admit_private_hosts_or_credentials() {
    let client = MediaHttpClient::public().expect("public media client");
    for url in [
        "https://127.0.0.1:27892/video.mp4",
        "https://192.168.1.1:27892/video.mp4",
        "https://[::1]:27892/video.mp4",
        "https://169.254.169.254:27892/video.mp4",
        "https://user:password@example.com:27892/video.mp4",
        "https://example.com:0/video.mp4",
    ] {
        assert!(client.get(url).is_err(), "unsafe URL accepted: {url}");
    }
}
