use super::definitive_capability_failure;

#[test]
fn malformed_track_and_unsupported_decoder_require_automatic_fallback() {
    assert!(definitive_capability_failure(Some("invalidVideoTrack")));
    assert!(definitive_capability_failure(Some("decoderUnsupported")));
    assert!(!definitive_capability_failure(Some("initialization")));
    assert!(!definitive_capability_failure(Some("runtimePlayback")));
    assert!(!definitive_capability_failure(None));
}
