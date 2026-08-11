use crate::manager::plan::eviction::{
    protected_seed_eviction, EvictionInputs, ProtectedSeedEviction,
};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::ByteRange;

static PRESENT: [ByteRange; 2] = [
    ByteRange { start: 0, end: 40 },
    ByteRange { start: 40, end: 80 },
];

#[test]
fn only_safe_contiguous_read_ahead_defers_protected_seed_eviction() {
    let healthy = inputs(PlaybackPhase::Playing);
    assert_eq!(
        protected_seed_eviction(healthy),
        ProtectedSeedEviction::Defer
    );
    assert_eq!(
        protected_seed_eviction(EvictionInputs {
            phase: Some(PlaybackPhase::Starting),
            playback_emergency: true,
            ..healthy
        }),
        ProtectedSeedEviction::Defer
    );
    for critical in [
        EvictionInputs {
            gateway_demand: false,
            ..healthy
        },
        EvictionInputs {
            current_startable: false,
            ..healthy
        },
        EvictionInputs {
            demanded: Some(ByteRange::new(96, 104)),
            ..healthy
        },
        EvictionInputs {
            buffer_below_emergency: true,
            ..healthy
        },
        EvictionInputs {
            phase: Some(PlaybackPhase::NetworkStalled),
            ..healthy
        },
        EvictionInputs {
            playback_emergency: true,
            ..healthy
        },
    ] {
        assert_eq!(
            protected_seed_eviction(critical),
            ProtectedSeedEviction::Allow
        );
    }
}

fn inputs(phase: PlaybackPhase) -> EvictionInputs<'static> {
    EvictionInputs {
        gateway_demand: true,
        current_startable: true,
        demanded: Some(ByteRange::new(80, 88)),
        present: &PRESENT,
        phase: Some(phase),
        playback_emergency: false,
        buffer_below_emergency: false,
    }
}
