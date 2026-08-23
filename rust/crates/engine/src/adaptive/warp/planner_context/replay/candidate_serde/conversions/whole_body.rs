use crate::adaptive::WholeBodyExhaustion;

pub(super) fn whole_body_exhaustion(
    maximum_bytes: Option<u64>,
    observed_bytes: Option<u64>,
) -> Option<WholeBodyExhaustion> {
    let maximum_bytes = maximum_bytes?;
    let observed_bytes = observed_bytes.or_else(|| maximum_bytes.checked_add(1))?;
    WholeBodyExhaustion::new(maximum_bytes, observed_bytes)
}
