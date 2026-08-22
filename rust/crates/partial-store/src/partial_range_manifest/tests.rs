use super::RangeManifest;

const ABOVE_WARP_BLOCK_LIMIT: u64 = 512 * 1024 + 1;

#[test]
fn encoder_never_persists_a_manifest_its_decoder_will_reject() {
    let mut manifest = RangeManifest::default();
    manifest.insert(0..ABOVE_WARP_BLOCK_LIMIT).unwrap();
    manifest
        .record_checksum(0..ABOVE_WARP_BLOCK_LIMIT, "0".repeat(64))
        .unwrap();

    assert!(manifest.to_json().is_err());
}
