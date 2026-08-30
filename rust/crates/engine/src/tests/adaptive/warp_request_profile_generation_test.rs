use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{ActionKind, AdaptivePlayabilityPolicy, MediaLayout, PlannerContext};
use crate::origin_model::{MediaClass, OriginRequestProfile, RequestMethod};
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::ByteRange;
use std::collections::BTreeSet;

#[test]
fn planner_commands_preserve_the_exact_forecast_request_profile() {
    let generated = generated_actions();
    let mut methods = BTreeSet::new();

    for action in generated.actions {
        match &action.node.kind {
            ActionKind::Head => {
                methods.insert(RequestMethod::Head);
                assert_profile(action.node.request_profile(), RequestMethod::Head, 0);
            }
            ActionKind::Prefix(range) => {
                methods.insert(RequestMethod::PrefixGet);
                assert_profile(
                    action.node.request_profile(),
                    RequestMethod::PrefixGet,
                    range.len(),
                );
            }
            ActionKind::Tail(range) => {
                methods.insert(RequestMethod::TailGet);
                assert_profile(
                    action.node.request_profile(),
                    RequestMethod::TailGet,
                    range.len(),
                );
            }
            ActionKind::FetchRange(range) => {
                methods.insert(RequestMethod::RangeGet);
                assert_profile(
                    action.node.request_profile(),
                    RequestMethod::RangeGet,
                    range.len(),
                );
            }
            ActionKind::FetchWhole { maximum_bytes } => {
                methods.insert(RequestMethod::FullGet);
                assert_profile(
                    action.node.request_profile(),
                    RequestMethod::FullGet,
                    *maximum_bytes,
                );
            }
            _ => {}
        }
    }

    assert_eq!(
        methods,
        BTreeSet::from([
            RequestMethod::Head,
            RequestMethod::PrefixGet,
            RequestMethod::TailGet,
            RequestMethod::RangeGet,
            RequestMethod::FullGet,
        ])
    );
}

fn generated_actions() -> crate::adaptive::GeneratedActions {
    let mut input = snapshot(2, 8_000_000, 1_000, 20);
    let observed_at_ms = input.observed_at_ms;
    input.candidates[1].layout = MediaLayout::Unknown;
    let candidate = &mut input.candidates[0];
    candidate.layout = MediaLayout::Unknown;
    set_reliable_total_bytes(candidate, 800_000, observed_at_ms);
    candidate.timeline_probe = Some(crate::adaptive::PlayableRange {
        bytes: ByteRange::new(736_000, 800_000),
        playable_ms: 0,
    });
    let base = AdaptivePlayabilityPolicy.plan(&input);
    WarpActionGenerator::generate(
        &input,
        &base,
        &crate::origin_model::OriginModel::default(),
        &PlannerContext::explicitly_unavailable(&input),
    )
}

fn assert_profile(actual: Option<OriginRequestProfile>, method: RequestMethod, bytes: u64) {
    assert_eq!(
        actual,
        Some(OriginRequestProfile::new(
            method,
            bytes,
            MediaClass::Unknown
        ))
    );
}
