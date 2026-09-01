use super::{MediaClass, NetworkClass, OriginContext, RequestMethod};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OriginRequestProfile {
    method: RequestMethod,
    planned_bytes: u64,
    media: MediaClass,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OpenBodyProfile {
    method: RequestMethod,
    media: MediaClass,
}

impl OpenBodyProfile {
    pub(crate) const fn from_request(profile: OriginRequestProfile) -> Self {
        Self {
            method: profile.method,
            media: profile.media,
        }
    }

    pub(crate) const fn request_profile(self, body_bytes: u64) -> OriginRequestProfile {
        OriginRequestProfile::new(self.method, body_bytes, self.media)
    }
}

impl OriginRequestProfile {
    pub const fn new(method: RequestMethod, planned_bytes: u64, media: MediaClass) -> Self {
        Self {
            method,
            planned_bytes,
            media,
        }
    }

    pub const fn method(self) -> RequestMethod {
        self.method
    }

    pub(crate) const fn planned_bytes(self) -> u64 {
        self.planned_bytes
    }

    pub const fn media(self) -> MediaClass {
        self.media
    }

    const fn with_transport(self, method: RequestMethod, planned_bytes: u64) -> Self {
        Self::new(method, planned_bytes, self.media)
    }

    pub fn context(self) -> OriginContext {
        OriginContext::new(self.method, self.planned_bytes, self.media)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OriginAttemptProfile {
    forecast: OriginRequestProfile,
    request: OriginRequestProfile,
    #[serde(default)]
    admission_intent: super::OriginAdmissionIntent,
}

impl OriginAttemptProfile {
    pub const fn new(forecast: OriginRequestProfile) -> Self {
        Self {
            forecast,
            request: forecast,
            admission_intent: super::OriginAdmissionIntent::Delivery,
        }
    }

    pub const fn forecast(self) -> OriginRequestProfile {
        self.forecast
    }

    pub const fn request(self) -> OriginRequestProfile {
        self.request
    }

    pub const fn admission_intent(self) -> super::OriginAdmissionIntent {
        self.admission_intent
    }

    pub const fn with_admission_intent(mut self, intent: super::OriginAdmissionIntent) -> Self {
        self.admission_intent = intent;
        self
    }

    pub const fn with_executed_transport(
        mut self,
        method: RequestMethod,
        planned_bytes: u64,
    ) -> Self {
        self.request = self.request.with_transport(method, planned_bytes);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OriginAttemptContext {
    profile: OriginAttemptProfile,
    request_context: OriginContext,
}

impl OriginAttemptContext {
    pub fn new(
        profile: OriginAttemptProfile,
        network: NetworkClass,
        concurrency: usize,
        started_at_ms: u64,
    ) -> Self {
        let request_context = profile
            .request()
            .context()
            .with_network(network)
            .with_concurrency(concurrency)
            .with_observed_at_ms(started_at_ms);
        Self {
            profile,
            request_context,
        }
    }

    pub const fn profile(self) -> OriginAttemptProfile {
        self.profile
    }

    pub const fn request_context(self) -> OriginContext {
        self.request_context
    }
}
