pub(crate) fn observed_admitted_capacity(
    progressive: usize,
    adaptive: usize,
    ceiling: usize,
) -> usize {
    progressive.max(adaptive).min(ceiling.max(1))
}

pub(crate) fn observed_claimed_requests(progressive: usize, segmented: usize) -> usize {
    progressive.saturating_add(segmented)
}
