use super::prepared_authority_fixture::PreparedAuthorityFixture;

const ROOT: &str = "https://media.example/index.m3u8";
const BODY: &[u8] = b"#EXTM3U\n#EXT-X-ENDLIST\n";

#[test]
fn identical_bytes_receive_a_new_revision_after_invalidation_and_reseed() {
    let fixture = PreparedAuthorityFixture::new(ROOT, true);
    let cached_generation = fixture.publish(1, BODY);
    let initial = fixture.authority();

    assert!(fixture.cache.invalidate_generation(ROOT, cached_generation));
    assert!(fixture.cache.snapshot("post").authority.is_none());
    assert!(!fixture.cache.accepts_prepared_authority(&initial));

    let reseeded_generation = fixture.publish(1, BODY);
    let reseeded = fixture.authority();

    assert_eq!(reseeded_generation, cached_generation);
    assert!(reseeded.asset_revision() > initial.asset_revision());
    assert!(fixture.cache.accepts_prepared_authority(&reseeded));
    assert!(!fixture.cache.accepts_prepared_authority(&initial));
}
