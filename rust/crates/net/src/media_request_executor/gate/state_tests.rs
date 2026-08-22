use super::GateState;
use crate::media_request_executor::gate::MediaRequestGate;
use crate::media_request_executor::MediaRequestLimits;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use tokio::sync::oneshot;

#[test]
fn coordinator_reserves_same_authority_capacity() {
    let mut fixture = Fixture::new(3, 2);
    fixture.enqueue("same.example", PreemptionAuthority::Speculative);
    assert_eq!(fixture.grant_count(), 1);
    fixture.enqueue("same.example", PreemptionAuthority::Transition);
    assert_eq!(fixture.grant_count(), 0);
    fixture.enqueue("same.example", PreemptionAuthority::PlaybackCritical);
    assert_eq!(fixture.grant_count(), 1);
}

#[test]
fn coordinator_reserves_global_capacity_across_authorities() {
    let mut fixture = Fixture::new(2, 1);
    fixture.enqueue("one.example", PreemptionAuthority::Transition);
    assert_eq!(fixture.grant_count(), 1);
    fixture.enqueue("two.example", PreemptionAuthority::Speculative);
    assert_eq!(fixture.grant_count(), 0);
    fixture.enqueue("three.example", PreemptionAuthority::PlaybackCritical);
    assert_eq!(fixture.grant_count(), 1);
}

#[test]
fn coordinator_does_not_reserve_the_only_slot() {
    let mut fixture = Fixture::new(1, 1);
    fixture.enqueue("one.example", PreemptionAuthority::Speculative);
    assert_eq!(fixture.grant_count(), 1);
}

#[test]
fn critical_occupancy_leaves_ordinary_capacity_available() {
    let mut fixture = Fixture::new(2, 2);
    fixture.enqueue("same.example", PreemptionAuthority::PlaybackCritical);
    assert_eq!(fixture.grant_count(), 1);
    fixture.enqueue("same.example", PreemptionAuthority::Speculative);
    assert_eq!(fixture.grant_count(), 1);
}

struct Fixture {
    state: GateState,
    gate: MediaRequestGate,
}

impl Fixture {
    fn new(global: usize, per_authority: usize) -> Self {
        let limits = MediaRequestLimits::try_new(global, per_authority).unwrap();
        Self {
            state: GateState::new(limits),
            gate: MediaRequestGate::new(limits),
        }
    }

    fn enqueue(&mut self, host: &str, priority: PreemptionAuthority) {
        let authority = RequestAuthority::from_url(&format!("https://{host}/media")).unwrap();
        let (sender, _receiver) = oneshot::channel();
        self.state.enqueue(authority, priority, sender);
    }

    fn grant_count(&mut self) -> usize {
        let grants = self.state.take_grants(&self.gate);
        let count = grants.len();
        for (sender, mut lease) in grants {
            drop(sender);
            assert!(lease.armed);
            lease.armed = false;
        }
        count
    }
}
