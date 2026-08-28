//! Non-stationary, method- and context-specific transport evidence for WARP §7.3–7.4.

mod admission_intent;
mod change;
mod circuit;
mod context;
mod environment;
mod errors;
mod estimate;
mod exploration;
mod hierarchy;
mod keys;
mod map_serde;
mod model;
mod observation;
mod open_body_observation;
mod prior;
mod probability;
mod quantile;
mod record;
mod request_profile;
mod retention;
mod timing;

pub use admission_intent::OriginAdmissionIntent;
pub use context::{
    ConcurrencyBucket, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
    SizeBucket, TimeOfDay,
};
pub use environment::{Availability, DomainClass, HttpProtocol, OriginEnvironment, TlsVersion};
pub use errors::ErrorReason;
pub use estimate::{
    AdaptationState, DecisionMode, OriginEstimate, ProbabilityEstimate, QuantileEstimate,
};
pub use exploration::ExplorationClaim;
pub use model::{
    Admission, AdmissionBlockReason, AdmissionClaim, AdmissionClaimTerminal, ClaimedAdmission,
    OriginModel,
};
pub use observation::{OriginObservation, OriginOutcome};
pub use open_body_observation::OpenBodyObservation;
pub use prior::{ColdStartPrior, ColdStartSelector};
pub use request_profile::{
    OpenBodyProfile, OriginAttemptContext, OriginAttemptProfile, OriginRequestProfile,
};

use prior::PriorRegistration;
use record::{AdaptiveRecord, RecordSnapshot};
use timing::ModelTiming;
