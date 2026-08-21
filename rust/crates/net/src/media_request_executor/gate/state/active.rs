use ghostr_engine::adaptive::PreemptionAuthority;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct ActiveCapacity {
    total: usize,
    ordinary: usize,
}

impl ActiveCapacity {
    pub(super) const fn total(self) -> usize {
        self.total
    }

    pub(super) fn available(self, limit: usize, priority: PreemptionAuthority) -> bool {
        self.total < limit && (is_critical(priority) || self.ordinary < ordinary_limit(limit))
    }

    pub(super) fn claim(&mut self, priority: PreemptionAuthority) {
        self.total += 1;
        if !is_critical(priority) {
            self.ordinary += 1;
        }
    }

    pub(super) fn release(&mut self, priority: PreemptionAuthority) {
        if is_critical(priority) {
            assert!(
                self.total > self.ordinary,
                "critical request capacity underflow"
            );
        } else {
            self.ordinary = decrement(self.ordinary, "ordinary request capacity underflow");
        }
        self.total = decrement(self.total, "active request capacity underflow");
    }
}

const fn ordinary_limit(limit: usize) -> usize {
    if limit > 1 {
        limit - 1
    } else {
        limit
    }
}

const fn is_critical(priority: PreemptionAuthority) -> bool {
    matches!(priority, PreemptionAuthority::PlaybackCritical)
}

fn decrement(value: usize, message: &'static str) -> usize {
    value.checked_sub(1).expect(message)
}
