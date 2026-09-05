use anyhow::{ensure, Context as _, Result};
use reqwest::{header::LOCATION, Response, StatusCode, Url};

pub(super) fn visit_key(url: &Url) -> Url {
    let mut key = url.clone();
    key.set_fragment(None);
    key
}

pub(super) fn redirect_target(response: &Response) -> Result<Option<Url>> {
    if !followed_status(response.status()) {
        return Ok(None);
    }
    let Some(location) = response.headers().get(LOCATION) else {
        return Ok(None);
    };
    let location = location.to_str().context("redirect Location is not text")?;
    let target = response
        .url()
        .join(location)
        .context("redirect Location is invalid")?;
    ensure!(
        target.username().is_empty() && target.password().is_none(),
        "media redirect credentials are forbidden"
    );
    Ok(Some(target))
}

fn followed_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}
