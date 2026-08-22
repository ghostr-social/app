use base64::prelude::*;

use super::fixture_expansion::expand_avc_samples;
use super::fixture_timing::scale_avc_timing;

const EXPANDED_SAMPLE_BYTES: usize = 9_446;
const TIMING_MULTIPLIER: u32 = 2;

pub(crate) fn progressive_mp4() -> Vec<u8> {
    scale_avc_timing(
        expand_avc_samples(source_mp4(), EXPANDED_SAMPLE_BYTES),
        TIMING_MULTIPLIER,
    )
}

pub(super) fn tail_moov_mp4() -> Vec<u8> {
    super::fixture_tail::move_moov_to_tail(progressive_mp4())
}

fn source_mp4() -> Vec<u8> {
    let source = include_str!("../../../../../../tool/video_user_e2e/media_fixture.mjs");
    let encoded = source
        .split("const MP4_BASE64 = \"")
        .nth(1)
        .expect("fixture prefix")
        .split("\";")
        .next()
        .expect("fixture suffix");
    BASE64_STANDARD
        .decode(encoded)
        .expect("progressive MP4 fixture")
}
