use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn rejects_literal_non_public_destinations() {
    let client = MediaHttpClient::public().expect("media client");
    for address in include_str!("../../../../test/support/public_media_private_addresses.txt").lines() {
        let url = media_url(address);
        assert!(client.get(&url).is_err(), "accepted {url}");
    }
}

fn media_url(address: &str) -> String {
    if address.contains(':') {
        format!("http://[{address}]/video.mp4")
    } else {
        format!("http://{address}/video.mp4")
    }
}
