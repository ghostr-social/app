//! Monotonic identity of one account-scoped Nostr session.

/// Token captured by asynchronous work before it leaves for relays.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SessionGeneration(u64);

pub const SESSION_RESET_MESSAGE: &str = "the Nostr session was reset";

impl SessionGeneration {
    pub const fn initial() -> Self {
        Self(0)
    }

    pub const fn from_value(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("Nostr session generation exhausted"),
        )
    }
}
