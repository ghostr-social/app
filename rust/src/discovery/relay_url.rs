//! Relay URL validation for outbox routing. Mirrors the observable
//! composition the Dart app applies to NIP-65 urls: ndk's
//! `cleanRelayUrl` followed by `RelayUrl.tryParse`
//! (lib/features/settings/domain/relay_url.dart) — wss:// anywhere,
//! ws:// only on local development hosts, no query or fragment,
//! userinfo stripped, lowercase scheme and host, default ports and
//! trailing slashes removed.

// `::1` also appears in the Dart policy but is unreachable there too:
// ndk's regex validation rejects IPv6 hosts before the policy runs.
const LOCAL_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];

/// The validated, normalized relay URL, or `None` when the raw value
/// is not an acceptable relay endpoint.
pub fn normalize_relay_url(raw: &str) -> Option<String> {
    let (scheme, rest) = split_scheme(raw.trim())?;
    if rest.contains('?') || rest.contains('#') {
        return None;
    }
    let (authority, path) = split_authority(rest);
    let (host, port) = split_host_port(strip_userinfo(authority))?;
    if !scheme_allows_host(&scheme, &host) {
        return None;
    }
    Some(assemble(&scheme, &host, without_default_port(&scheme, port), path))
}

fn split_scheme(raw: &str) -> Option<(String, &str)> {
    let (scheme, rest) = raw.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "ws" && scheme != "wss" {
        return None;
    }
    // ndk repairs extra slashes after the scheme (wss:/// -> wss://).
    Some((scheme, rest.trim_start_matches('/')))
}

fn split_authority(rest: &str) -> (&str, &str) {
    match rest.find('/') {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    }
}

// ndk rebuilds the URL without userinfo, so credentials never reach
// the strict policy: they are dropped, not rejected.
fn strip_userinfo(authority: &str) -> &str {
    authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host)
}

fn split_host_port(authority: &str) -> Option<(String, Option<u16>)> {
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port.parse::<u16>().ok()?)),
        None => (authority, None),
    };
    let host = host.to_ascii_lowercase();
    host_is_valid(&host).then_some((host, port))
}

fn host_is_valid(host: &str) -> bool {
    !host.is_empty()
        && !host.starts_with('-')
        && !host.ends_with('-')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

fn scheme_allows_host(scheme: &str, host: &str) -> bool {
    scheme == "wss" || LOCAL_HOSTS.contains(&host)
}

fn without_default_port(scheme: &str, port: Option<u16>) -> Option<u16> {
    port.filter(|&port| !is_default_port(scheme, port))
}

fn is_default_port(scheme: &str, port: u16) -> bool {
    (scheme == "wss" && port == 443) || (scheme == "ws" && port == 80)
}

fn assemble(scheme: &str, host: &str, port: Option<u16>, path: &str) -> String {
    let port = port.map(|port| format!(":{port}")).unwrap_or_default();
    format!("{scheme}://{host}{port}{}", path.trim_end_matches('/'))
}
