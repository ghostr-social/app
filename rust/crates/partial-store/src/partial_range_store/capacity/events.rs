use tokio::sync::watch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityRevision(u64);

impl CapacityRevision {
    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CapacityEvents {
    changes: watch::Sender<u64>,
}

impl CapacityEvents {
    pub(super) fn new() -> Self {
        let (changes, _) = watch::channel(0);
        Self { changes }
    }

    pub(crate) fn signal(&self) {
        self.changes
            .send_modify(|generation| *generation = generation.saturating_add(1));
    }

    pub(crate) fn revision(&self) -> CapacityRevision {
        CapacityRevision(*self.changes.borrow())
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<u64> {
        self.changes.subscribe()
    }
}
