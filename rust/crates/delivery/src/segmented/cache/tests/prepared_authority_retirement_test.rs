use super::prepared_authority_fixture::PreparedAuthorityFixture;
use crate::segmented::cache::objects;

const FIRST: &str = "https://first.example/index.m3u8";
const SECOND: &str = "https://second.example/index.m3u8";
const BODY: &[u8] = b"#EXTM3U\n";

#[test]
fn every_ready_retirement_path_revokes_its_asset_authority() {
    let mut fixture = PreparedAuthorityFixture::new(FIRST, true);
    fixture.publish(1, BODY);
    let initial = fixture.authority();

    assert!(fixture.cache.reset_stage_retry(&fixture.post, 1));
    assert_revoked(&fixture, &initial);
    fixture.publish(1, BODY);
    let after_reset = fixture.authority();
    assert!(after_reset.asset_revision() > initial.asset_revision());

    fixture.replace_focus(2, SECOND, true);
    assert_revoked(&fixture, &after_reset);
    fixture.publish(2, BODY);
    let after_replacement = fixture.authority();
    assert!(after_replacement.asset_revision() > after_reset.asset_revision());

    fixture.cache.clear();
    assert_revoked(&fixture, &after_replacement);
    fixture.replace_focus(3, SECOND, false);
    fixture.publish(3, BODY);
    let after_clear = fixture.authority();
    assert!(after_clear.asset_revision() > after_replacement.asset_revision());

    objects::reclaim_unprotected_ready(&mut fixture.cache.lock());
    assert_revoked(&fixture, &after_clear);
}

fn assert_revoked(
    fixture: &PreparedAuthorityFixture,
    authority: &crate::segmented::HlsPreparedAssetAuthority,
) {
    assert!(fixture.cache.snapshot("post").authority.is_none());
    assert!(!fixture.cache.accepts_prepared_authority(authority));
}
