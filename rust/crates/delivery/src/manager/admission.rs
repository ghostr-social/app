use ghostr_engine::RequestAuthority;

const INVALID_AUTHORITY: &str = "invalid-request-authority";

pub(crate) fn origin_key(url: &str) -> String {
    RequestAuthority::from_url(url)
        .map(|authority| authority.as_str().to_owned())
        .unwrap_or_else(|| INVALID_AUTHORITY.to_owned())
}
