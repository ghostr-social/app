use super::player_preparation_manager_fixture::ProductionManagerFixture;
use crate::api::delivery_types::{FfiPlayerPreparationDisposition, FfiPlayerPreparationState};
use crate::api::player_preparation_control::confirm_player_preparation;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn production_manager_confirms_evidence_and_authority_withdrawal() {
    let fixture = ProductionManagerFixture::seeded().await;
    let input = fixture.input.clone();

    let applied = timeout(
        Duration::from_secs(2),
        confirm_player_preparation(&fixture.context, input.clone()),
    )
    .await
    .expect("production manager confirmation");
    assert_eq!(applied, FfiPlayerPreparationDisposition::Applied);
    assert_eq!(
        confirm_player_preparation(&fixture.context, input).await,
        FfiPlayerPreparationDisposition::Duplicate,
    );
    assert_eq!(
        fixture
            .context
            .delivery
            .update_focus(DeliveryFocus::compatibility(Vec::new(), 0, 0)),
        FocusAdmission::Accepted,
    );
    let mut followup = fixture.input.clone();
    followup.sequence = 2;
    followup.state = FfiPlayerPreparationState::Initialized;
    let rejected = timeout(
        Duration::from_secs(2),
        confirm_player_preparation(&fixture.context, followup),
    )
    .await
    .expect("production manager rejection");
    assert_eq!(rejected, FfiPlayerPreparationDisposition::Rejected);
    fixture.shutdown().await;
}
