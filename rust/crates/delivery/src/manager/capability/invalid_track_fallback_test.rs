use super::fallback_fixture::FallbackFixture;

#[tokio::test]
async fn only_definitive_player_failures_switch_to_a_fallback_rendition() {
    for (failure, expected_switch) in [
        ("invalidVideoTrack", true),
        ("decoderUnsupported", true),
        ("initialization", false),
        ("runtimePlayback", false),
    ] {
        let mut fixture = FallbackFixture::new().await;
        fixture.report(1, None).await;
        fixture.report(2, Some(failure)).await;

        assert_eq!(fixture.selected_fallback(), expected_switch, "{failure}");
    }
}
