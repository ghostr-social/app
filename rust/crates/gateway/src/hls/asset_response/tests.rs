use super::{AssetBodyContract, AssetResponseEnvelope};
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;

#[test]
fn body_contracts_bound_unknown_lengths_and_require_exact_extents() {
    let capped = AssetResponseEnvelope::Full { length: None }.body_contract();
    assert_eq!(
        capped,
        AssetBodyContract::Capped {
            maximum: MAX_HLS_ASSET_BYTES as u64
        }
    );
    assert_eq!(
        capped.checked_total(0, MAX_HLS_ASSET_BYTES),
        Some(MAX_HLS_ASSET_BYTES as u64)
    );
    assert_eq!(capped.checked_total(MAX_HLS_ASSET_BYTES as u64, 1), None);
    assert!(capped.complete(3));

    let exact = AssetBodyContract::Exact { bytes: 4 };
    assert_eq!(exact.checked_total(0, 4), Some(4));
    assert_eq!(exact.checked_total(4, 1), None);
    assert!(!exact.complete(2));
    assert!(exact.complete(4));
}
