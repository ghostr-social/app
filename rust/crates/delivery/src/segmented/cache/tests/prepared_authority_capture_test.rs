use super::prepared_authority_fixture::PreparedAuthorityFixture;

const ROOT: &str = "https://media.example/index.m3u8";
const BODY: &[u8] = b"#EXTM3U\n";

#[test]
fn exact_authority_atomically_pins_its_ready_object_cohort() {
    let fixture = PreparedAuthorityFixture::new(ROOT, true);
    let generation = fixture.publish(1, BODY);
    let authority = fixture.authority();
    let captured = fixture
        .cache
        .capture_prepared_asset(&authority, &[ROOT.to_owned()])
        .expect("current exact prepared asset");

    assert_eq!(captured.authority(), &authority);
    assert_eq!(captured.root_source(), ROOT);
    assert_eq!(captured.playback_manifest_source(), ROOT);
    assert_eq!(
        captured.object(ROOT).expect("pinned root").body.as_ref(),
        BODY
    );
    assert!(fixture
        .cache
        .capture_prepared_asset(&authority, &["https://other.example/root.m3u8".to_owned()])
        .is_none());

    assert!(fixture.cache.invalidate_generation(ROOT, generation));
    assert!(fixture
        .cache
        .capture_prepared_asset(&authority, &[ROOT.to_owned()])
        .is_none());
    assert_eq!(
        captured.object(ROOT).expect("retained root").body.as_ref(),
        BODY
    );
}
