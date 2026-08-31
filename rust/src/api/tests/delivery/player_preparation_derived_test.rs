use crate::api::playback_preparation_stream::projection;
use crate::api::player_preparation_control::axiom_test_support::report_player_preparation;
use crate::api::tests::delivery::player_preparation_authority_fixture::AuthorityFixture;
use ghostr_engine::PostId;

#[tokio::test]
async fn selected_derived_representation_keeps_feed_source_and_exact_player_authority() {
    let mut fixture = AuthorityFixture::seeded().await;
    let source_representation = fixture.publish_selected_representation().await;
    fixture.publish_derived_representation().await;
    fixture
        .commands
        .publish_focused_plan(7, Some(PostId::new("clip")), Default::default());

    let plan = projection::project(&fixture.preparation_context())
        .await
        .expect("derived preparation plan");
    let asset = plan.current.expect("derived current asset");
    assert_eq!(asset.representation_id, fixture.representation);
    assert_eq!(asset.source_representation_id, source_representation);
    assert_ne!(asset.source_representation_id, asset.representation_id);
    assert_eq!(asset.asset_id, fixture.asset);
    report_player_preparation(&fixture.context, fixture.input())
        .await
        .expect("test fixture precondition must hold");
    let report = fixture
        .commands
        .try_player_preparation()
        .expect("test fixture precondition must hold");
    assert_eq!(
        report
            .progressive_binding()
            .expect("progressive authority")
            .representation()
            .fingerprint(),
        fixture.representation
    );
}
