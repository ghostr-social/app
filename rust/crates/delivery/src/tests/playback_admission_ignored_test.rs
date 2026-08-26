use crate::playback_admission::{PlaybackAdmission, PlaybackAdmissionLedger};
use ghostr_engine::PostId;

#[test]
fn ignored_terminal_teardown_does_not_change_health_counters() {
    let ledger = PlaybackAdmissionLedger::default();

    ledger.record(PlaybackAdmission::IgnoredInactive, &PostId::new("retired"));

    let snapshot = ledger.snapshot();
    assert_eq!(snapshot.counters(), Default::default());
    assert_eq!(snapshot.last_accepted(), None);
}
