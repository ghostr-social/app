use ghostr_engine::origin_model::NetworkClass;
use std::time::Duration;

mod failure;
pub(in crate::segmented) use failure::FetchFailure;
mod problem;
pub(in crate::segmented::fetch) use problem::FetchProblem;
mod progress;
pub(in crate::segmented) use progress::FetchProgress;
mod traffic;
pub(in crate::segmented) use traffic::SegmentedTraffic;

#[derive(Clone, Copy, Debug)]
pub(in crate::segmented) struct OriginTelemetry {
    pub elapsed: Duration,
    pub ttfb: Option<Duration>,
    pub concurrency: usize,
    pub network_class: NetworkClass,
}
