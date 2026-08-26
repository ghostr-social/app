use super::*;

use ghostr_delivery::delivery_events::PlayerPreparationIngress;

pub(crate) async fn report_player_preparation(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    let initial = input.sequence == 1 && input.state == FfiPlayerPreparationState::Initializing;
    if initial {
        report_initial(context, input).await
    } else {
        report_followup(context, &input)
    }
}

async fn report_initial(
    context: &PlayerPreparationContext,
    input: FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    let admission = context.delivery.player_preparation_admission();
    let authority = validate_asset(context, &input).await?;
    let report = map_initial(&input, authority)?;
    admit(
        context
            .delivery
            .report_player_preparation_initial(admission, report),
    )
}

fn report_followup(
    context: &PlayerPreparationContext,
    input: &FfiPlayerPreparationReport,
) -> anyhow::Result<()> {
    admit(
        context
            .delivery
            .report_player_preparation_followup(map_followup(input)?),
    )
}

fn admit(admission: PlayerPreparationIngress) -> anyhow::Result<()> {
    match admission {
        PlayerPreparationIngress::Accepted | PlayerPreparationIngress::Duplicate => Ok(()),
        PlayerPreparationIngress::Stale
        | PlayerPreparationIngress::Rejected
        | PlayerPreparationIngress::InvalidAdmission
        | PlayerPreparationIngress::MissingInitial
        | PlayerPreparationIngress::Pending => {
            anyhow::bail!("player preparation attempt was not admitted")
        }
        PlayerPreparationIngress::Saturated => {
            anyhow::bail!("player preparation mailbox is saturated")
        }
        PlayerPreparationIngress::Closed => anyhow::bail!("delivery manager is unavailable"),
    }
}
