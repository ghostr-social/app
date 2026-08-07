use crate::native_cache_failure::permanent;
use anyhow::Result;
use reqwest::Url;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

const BLOCKED_IPV4: &[([u8; 4], u8)] = &[
    ([0, 0, 0, 0], 8),
    ([10, 0, 0, 0], 8),
    ([100, 64, 0, 0], 10),
    ([127, 0, 0, 0], 8),
    ([169, 254, 0, 0], 16),
    ([172, 16, 0, 0], 12),
    ([192, 0, 0, 0], 24),
    ([192, 0, 2, 0], 24),
    ([192, 88, 99, 0], 24),
    ([192, 168, 0, 0], 16),
    ([198, 18, 0, 0], 15),
    ([198, 51, 100, 0], 24),
    ([203, 0, 113, 0], 24),
    ([224, 0, 0, 0], 4),
    ([240, 0, 0, 0], 4),
];

pub fn validate_url(url: &Url) -> Result<()> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(permanent("media URL scheme is not allowed"));
    }
    let host = url
        .host_str()
        .ok_or_else(|| permanent("media URL host is missing"))?;
    if parse_ip(host).is_some_and(|address| !is_public(address)) {
        return Err(permanent("media URL targets a non-public address"));
    }
    Ok(())
}

fn parse_ip(host: &str) -> Option<IpAddr> {
    host.parse()
        .ok()
        .or_else(|| host.strip_prefix('[')?.strip_suffix(']')?.parse().ok())
}

pub fn is_public(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_v4(address),
        IpAddr::V6(address) => is_public_v6(address),
    }
}

fn is_public_v4(address: Ipv4Addr) -> bool {
    !BLOCKED_IPV4
        .iter()
        .any(|(network, prefix)| contains_v4(address, *network, *prefix))
}

fn is_public_v6(address: Ipv6Addr) -> bool {
    if let Some(mapped) = address.to_ipv4_mapped() {
        return is_public_v4(mapped);
    }
    let value = u128::from(address);
    contains(value, 0x2000 << 112, 3)
        && !contains(value, 0x2001 << 112, 23)
        && !contains(value, 0x2001_0db8 << 96, 32)
        && !contains(value, 0x2002 << 112, 16)
        && !contains(value, 0x3ffe << 112, 16)
        && !contains(value, 0x3fff << 112, 20)
}

fn contains_v4(address: Ipv4Addr, network: [u8; 4], prefix: u8) -> bool {
    contains(
        u32::from(address) as u128,
        u32::from(Ipv4Addr::from(network)) as u128,
        prefix,
    )
}

fn contains(address: u128, network: u128, prefix: u8) -> bool {
    let width = if address <= u32::MAX as u128 { 32 } else { 128 };
    let mask = u128::MAX << (width - prefix);
    (address & mask) == network
}
