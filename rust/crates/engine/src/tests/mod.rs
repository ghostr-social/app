//! Shared fixtures sit here; each folder holds the tests
//! for the matching folder of the crate.

pub(crate) mod adaptive_support;
mod byte_range_test;
mod cmaf_timeline_support;
mod host_stats_support;
mod media_timeline_assertions;
mod media_timeline_support;
mod rendition_support;
mod request_authority_test;
mod support;

mod adaptive;
mod budget;
mod catalog;
mod concurrency;
mod focus;
mod host_stats;
mod media_timeline;
mod playback;
mod rendition;
mod video_rendition_validation_test;
