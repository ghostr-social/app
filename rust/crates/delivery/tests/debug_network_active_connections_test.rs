use ghostr_delivery::debug::network::NetworkThrottle;

#[test]
fn active_connections_reports_sorted_counts_and_releases() {
    let throttle = NetworkThrottle::default();
    let beta_first = throttle.acquire("https://beta.example/first.mp4");
    let alpha = throttle.acquire("https://alpha.example/video.mp4");
    let beta_second = throttle.acquire("https://beta.example/second.mp4");

    assert_eq!(
        throttle.active_connections(),
        vec![
            ("https://alpha.example".to_owned(), 1),
            ("https://beta.example".to_owned(), 2),
        ]
    );

    drop((beta_first, alpha, beta_second));
    assert!(throttle.active_connections().is_empty());
}
