use super::prepared_authority_fixture::PreparedAuthorityFixture;

const ROOT: &str = "https://media.example/index.m3u8";

#[test]
fn ready_publication_owns_a_typed_stable_asset_authority() {
    let mut fixture = PreparedAuthorityFixture::new(ROOT, true);
    assert!(fixture.cache.snapshot("post").authority.is_none());

    fixture.publish(1, b"#EXTM3U\n");
    let initial = fixture.authority();
    assert_eq!(initial.post(), &fixture.post);
    assert_eq!(initial.representation_id(), &fixture.representation);
    assert!(fixture.cache.accepts_prepared_authority(&initial));

    fixture.replace_focus(2, ROOT, true);

    assert_eq!(fixture.authority(), initial);
    assert!(fixture.cache.accepts_prepared_authority(&initial));
}
