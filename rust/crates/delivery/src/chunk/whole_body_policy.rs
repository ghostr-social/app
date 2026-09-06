pub(crate) fn is(error: &anyhow::Error) -> bool {
    error.is::<super::body_lease::BodyLeaseDenied>()
        || super::whole_body_limit::from_error(error).is_some()
        || super::whole_body_bound::from_error(error).is_some()
}
