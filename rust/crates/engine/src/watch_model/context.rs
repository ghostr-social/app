use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WatchKey(String);

impl WatchKey {
    pub fn digest(raw: &str) -> Self {
        Self(format!("{:x}", Sha256::digest(raw.as_bytes())))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DurationBucket {
    Unknown,
    Tiny,
    Short,
    Medium,
    Long,
    Extended,
}

impl DurationBucket {
    pub(super) fn of(duration_ms: Option<u64>) -> Self {
        match duration_ms {
            None => Self::Unknown,
            Some(0..=3_000) => Self::Tiny,
            Some(3_001..=10_000) => Self::Short,
            Some(10_001..=30_000) => Self::Medium,
            Some(30_001..=90_000) => Self::Long,
            Some(_) => Self::Extended,
        }
    }

    pub(super) fn neighbors(self) -> impl Iterator<Item = Self> {
        let values = match self {
            Self::Unknown => [None, None],
            Self::Tiny => [None, Some(Self::Short)],
            Self::Short => [Some(Self::Tiny), Some(Self::Medium)],
            Self::Medium => [Some(Self::Short), Some(Self::Long)],
            Self::Long => [Some(Self::Medium), Some(Self::Extended)],
            Self::Extended => [Some(Self::Long), None],
        };
        values.into_iter().flatten()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchContext {
    pub(super) video: WatchKey,
    pub(super) creator: Option<WatchKey>,
    pub(super) categories: Vec<WatchKey>,
    pub(super) user: Option<WatchKey>,
    pub(super) duration_ms: Option<u64>,
}

impl WatchContext {
    pub fn new(video: WatchKey, duration_ms: Option<u64>) -> Self {
        Self {
            video,
            creator: None,
            categories: Vec::new(),
            user: None,
            duration_ms,
        }
    }
}

#[cfg(test)]
#[path = "context/test_support.rs"]
mod test_support;
