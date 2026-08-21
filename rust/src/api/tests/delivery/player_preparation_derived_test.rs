use crate::api::playback_preparation_stream::projection;
use crate::api::player_preparation_control::report_player_preparation;
use crate::api::tests::delivery::player_preparation_authority_fixture::AuthorityFixture;
use ghostr_engine::PostId;

#[tokio::test]
async fn derived_representation_is_projected_and_accepted_as_exact_player_authority() {
    let mut fixture = AuthorityFixture::seeded().await;
    fixture.publish_derived_representation().await;
    fixture
        .commands
        .publish_focused_plan(7, Some(PostId::new("clip")), Default::default());

    let plan = projection::project(&fixture.preparation_context())
        .await
        .expect("derived preparation plan");
    let asset = plan.current.expect("derived current asset");
    assert_eq!(asset.representation_id, fixture.representation);
    assert_eq!(asset.asset_id, fixture.asset);
    report_player_preparation(&fixture.context, fixture.input())
        .await
        .unwrap();
    let report = fixture.commands.try_player_preparation().unwrap();
    assert_eq!(
        report.binding().representation().fingerprint(),
        fixture.representation
    );
}
