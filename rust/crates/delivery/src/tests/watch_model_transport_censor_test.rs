use crate::qoe::axiom_test_support::WatchOutcome;
use crate::delivery_events::{FocusTransition, TransportRescueReason};
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::{focus, rescue};
use ghostr_engine::watch_model::{WatchCensor, WatchSampleKind};

#[test]
fn system_substitutions_are_typed_censors_and_never_abandonments() {
    let rows = [
        (
            rescue(2, 900, TransportRescueReason::EtaTooLong),
            WatchCensor::TransportSubstitution,
        ),
        (
            rescue(2, 900, TransportRescueReason::DeliveryFailed),
            WatchCensor::OriginFailure,
        ),
        (
            focus(2, 900, FocusTransition::RosterChange),
            WatchCensor::PolicyRejection,
        ),
    ];
    for (departure, reason) in rows {
        let mut learner = WatchLearner::default();
        learner.focus(&focus(1, 0, FocusTransition::UserNavigation), 10);
        learner.focus(&departure, 20);
        assert_eq!(
            learner.last_outcome(),
            Some(WatchOutcome::sample(900, WatchSampleKind::Censored(reason),))
        );
        assert_eq!(learner.model().revision(), 0);
    }
}
