use ghostr_media_model::native_media_metadata::lenient_native_media;

fn imeta(bitrate: &str) -> Vec<String> {
    vec![
        "imeta".to_owned(),
        "url https://cdn.example/video.mp4".to_owned(),
        "m video/mp4".to_owned(),
        format!("bitrate {bitrate}"),
    ]
}

#[test]
fn native_media_metadata_keeps_only_positive_bit_rates_in_bits_per_second() {
    let valid = lenient_native_media(&imeta("3000000")).expect("valid media");
    assert_eq!(valid.extras.bitrate_bps, Some(3_000_000));

    for malformed in ["0", "-1", "2.5", "fast", "18446744073709551616"] {
        let media = lenient_native_media(&imeta(malformed)).expect("playable media");
        assert_eq!(media.extras.bitrate_bps, None, "{malformed}");
    }
}
