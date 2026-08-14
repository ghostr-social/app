//! Canonical relay URL policy for configured and NIP-65 values:
//! `wss` anywhere, `ws` only on local development hosts, no query or
//! fragment, no userinfo, lowercase scheme and host, and no default
//! ports or trailing slashes.

const LOCAL_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];
const PRIVATE_DOMAIN_SUFFIXES: [&str; 4] = [".internal", ".lan", ".local", ".localhost"];

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
    Some(assemble(
        &scheme,
        &host,
        without_default_port(&scheme, port),
        path,
    ))
}

/// NIP-18 hints are attacker-controlled network targets. They must use
/// public-domain WSS endpoints; configured relays retain the broader policy.
pub fn normalize_untrusted_relay_url(raw: &str) -> Option<String> {
    let normalized = normalize_relay_url(raw)?;
    let parsed = url::Url::parse(&normalized).ok()?;
    if parsed.scheme() != "wss" {
        return None;
    }
    match parsed.host()? {
        url::Host::Domain(host) if public_domain(host) => Some(normalized),
        _ => None,
    }
}

fn public_domain(host: &str) -> bool {
    host.contains('.')
        && !host.ends_with('.')
        && !PRIVATE_DOMAIN_SUFFIXES
            .iter()
            .any(|suffix| host.ends_with(suffix))
}

fn split_scheme(raw: &str) -> Option<(String, &str)> {
    let (scheme, rest) = raw.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "ws" && scheme != "wss" {
        return None;
    }
    // Repair extra slashes after the scheme (`wss:///` -> `wss://`).
    Some((scheme, rest.trim_start_matches('/')))
}

fn split_authority(rest: &str) -> (&str, &str) {
    match rest.find('/') {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    }
}

// Credentials are never part of a relay identity or network target.
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
    !has_invalid_edge(host) && host.chars().all(is_host_character)
}

fn has_invalid_edge(host: &str) -> bool {
    host.is_empty() || host.starts_with('-') || host.ends_with('-')
}

fn is_host_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-')
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
