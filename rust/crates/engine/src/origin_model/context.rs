use crate::host_stats::host_of;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RequestMethod {
    Head,
    PrefixGet,
    TailGet,
    RangeGet,
    FullGet,
    ManifestGet,
    SegmentGet,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SizeBucket {
    Empty,
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl SizeBucket {
    fn of(bytes: u64) -> Self {
        match bytes {
            0 => Self::Empty,
            1..=65_536 => Self::Tiny,
            65_537..=1_048_576 => Self::Small,
            1_048_577..=8_388_608 => Self::Medium,
            8_388_609..=67_108_864 => Self::Large,
            _ => Self::Huge,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MediaClass {
    Unknown,
    Segmented,
    FragmentedMp4,
    ProgressiveMp4,
    TailMoovRange,
    WholeObject,
    TransformRequired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum NetworkClass {
    Unavailable,
    Wifi,
    Cellular,
    Wired,
    Constrained,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TimeOfDay {
    Night,
    Morning,
    Afternoon,
    Evening,
}

impl TimeOfDay {
    fn at_utc_ms(at_ms: u64) -> Self {
        let hour = (at_ms / 3_600_000) % 24;
        match hour {
            0..=5 => Self::Night,
            6..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            _ => Self::Evening,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConcurrencyBucket {
    One,
    Two,
    ThreeOrFour,
    FiveOrMore,
}

impl ConcurrencyBucket {
    fn of(active: usize) -> Self {
        match active.max(1) {
            1 => Self::One,
            2 => Self::Two,
            3..=4 => Self::ThreeOrFour,
            _ => Self::FiveOrMore,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OriginContext {
    pub(super) method: RequestMethod,
    size: SizeBucket,
    media: MediaClass,
    pub network: NetworkClass,
    time_of_day: TimeOfDay,
    concurrency: ConcurrencyBucket,
}

impl OriginContext {
    pub fn new(method: RequestMethod, bytes: u64, media: MediaClass) -> Self {
        Self {
            method,
            size: SizeBucket::of(bytes),
            media,
            network: NetworkClass::Unavailable,
            time_of_day: TimeOfDay::Night,
            concurrency: ConcurrencyBucket::One,
        }
    }

    pub fn with_network(mut self, network: NetworkClass) -> Self {
        self.network = network;
        self
    }

    pub fn with_concurrency(mut self, active: usize) -> Self {
        self.concurrency = ConcurrencyBucket::of(active);
        self
    }

    pub fn with_observed_at_ms(mut self, at_ms: u64) -> Self {
        self.time_of_day = TimeOfDay::at_utc_ms(at_ms);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OriginQuery {
    url: String,
    origin: String,
    url_id: String,
    pub(super) context: OriginContext,
    pub(super) environment: super::OriginEnvironment,
}

impl OriginQuery {
    pub fn new(url: impl Into<String>, context: OriginContext) -> Self {
        let url = url.into();
        Self {
            origin: host_of(&url).unwrap_or_else(|| "unavailable".to_owned()),
            url_id: hashed_url(&url),
            url,
            context,
            environment: super::OriginEnvironment::unavailable(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub(super) fn origin(&self) -> &str {
        &self.origin
    }

    pub(super) fn url_id(&self) -> &str {
        &self.url_id
    }
}

#[cfg(test)]
#[path = "context/test_support.rs"]
mod test_support;

fn hashed_url(url: &str) -> String {
    use core::fmt::Write as _;

    let digest = Sha256::digest(url.as_bytes());
    let mut encoded = String::with_capacity(24);
    for byte in &digest[..12] {
        write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
