use crate::delivery_events::PlayerPreparationState;
use crate::tests::player_preparation_hls_fixture::HlsPreparationFixture;
use ghostr_engine::adaptive::PlayerPreparation;

#[test]
fn hls_decoder_readiness_is_projected_only_for_the_live_segmented_authority() {
    let mut fixture = HlsPreparationFixture::new();
    let report = fixture.report(PlayerPreparationState::FirstFrameRendered);
    assert!(fixture.segmented.accepts_player_preparation(&report));
    assert!(fixture.state.apply_player_preparation(report));
    assert_eq!(fixture.state.client_capability_revision(), 0);
    assert_eq!(projected(&fixture), PlayerPreparation::FirstFrameRendered);

    fixture.cache.clear();

    let retired = fixture.report(PlayerPreparationState::FirstFrameRendered);
    assert!(!fixture.segmented.accepts_player_preparation(&retired));
    assert_eq!(projected(&fixture), PlayerPreparation::Unverified);
}

fn projected(fixture: &HlsPreparationFixture) -> PlayerPreparation {
    fixture
        .segmented
        .planning_candidates(fixture.state.navigation(1), &fixture.state)
        .into_iter()
        .next()
        .expect("HLS planning candidate")
        .player_preparation
}
