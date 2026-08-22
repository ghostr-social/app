//! Bounded, privacy-minimized measurements for the WARP evaluation contract.

mod events;
mod latency;
mod ledger;
mod privacy;
mod tracker;
mod types;

pub use events::{
    AdaptationMetricEvent, BudgetMetricEvent, IntegrityMetricEvent, PlaybackMetricEvent,
    PresentationMetricEvent, ReadinessMetricEvent, SemanticMetricEvent, TransferMetricEvent,
};
pub(crate) use ledger::EvaluationLedger;
pub use tracker::EvaluationTracker;
pub use types::{
    AdaptationMetrics, BudgetMetrics, EfficiencyMetrics, EvaluationSnapshot, IntegrityMetrics,
    LatencyDistribution, ReadinessMetrics, SemanticsMetrics, UserVisibleMetrics,
};
