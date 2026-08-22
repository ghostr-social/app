use super::super::{FetchProblem, FetchSpec};
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::strong_etag::StrongEtag;
use reqwest::StatusCode;
use url::Url;

pub(super) fn validate_status(status: StatusCode, offset: u64) -> Result<(), FetchProblem> {
    if offset > 0 && status == StatusCode::RANGE_NOT_SATISFIABLE {
        return Err(FetchProblem::restart_object(
            anyhow::anyhow!("continued HLS range is no longer satisfiable"),
            ErrorReason::RangeNoncompliant,
        ));
    }
    Ok(())
}

pub(super) fn validate(
    final_url: &Url,
    spec: FetchSpec<'_>,
    total: u64,
    etag: Option<&StrongEtag>,
) -> Result<(), FetchProblem> {
    super::generation(
        spec.object.total.is_none_or(|known| known == total),
        "HLS object length changed",
    )?;
    super::generation(
        spec.object.final_url.is_none_or(|known| known == final_url),
        "HLS final URL changed",
    )?;
    super::generation(
        spec.object
            .strong_etag
            .is_none_or(|known| etag == Some(known)),
        "HLS strong ETag changed",
    )
}
