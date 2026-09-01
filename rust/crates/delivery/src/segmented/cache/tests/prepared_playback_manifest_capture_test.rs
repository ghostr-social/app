use super::prepared_authority_fixture::PreparedAuthorityFixture;

const ROOT: &str = "https://media.example/index.m3u8";
const CHILD: &str = "https://media.example/selected.m3u8";
const INIT: &str = "https://media.example/init.mp4";
const SEGMENT: &str = "https://media.example/first.m4s";

#[test]
fn prepared_asset_captures_the_scheduler_selected_media_playlist() {
    let fixture = PreparedAuthorityFixture::new(ROOT, true);
    for (url, body) in [
        (ROOT, b"#EXTM3U\nmaster".as_slice()),
        (CHILD, b"#EXTM3U\nmedia".as_slice()),
        (INIT, b"init".as_slice()),
        (SEGMENT, b"segment".as_slice()),
    ] {
        fixture.stage(1, url, body);
    }
    assert!(fixture
        .cache
        .mark_stage_ready_for_playback(&fixture.post, 1, CHILD));
    let authority = fixture.authority();
    let asset = fixture
        .cache
        .capture_prepared_asset(&authority, &[ROOT.to_owned()])
        .expect("prepared playback asset");

    assert_eq!(asset.root_source(), ROOT);
    assert_eq!(asset.playback_manifest_source(), CHILD);
    assert!(asset.object(CHILD).is_some());
}
