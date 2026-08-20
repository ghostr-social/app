//! NIP-B7/BUD-03 hash identity extracted from a media URL.

/// The last terminal 64-hex value in the final path segment, normalized.
/// A file extension may follow it, as allowed by NIP-B7.
pub fn terminal_sha256(raw: &str) -> Option<String> {
    let url = reqwest::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host().is_none() {
        return None;
    }
    let name = url.path_segments()?.next_back()?;
    terminal_hex(name.as_bytes()).map(|value| value.to_ascii_lowercase())
}

fn terminal_hex(name: &[u8]) -> Option<&str> {
    if name.len() < 64 {
        return None;
    }
    for start in (0..=name.len() - 64).rev() {
        let end = start + 64;
        let candidate = &name[start..end];
        if candidate.iter().all(u8::is_ascii_hexdigit)
            && boundary_before(name, start)
            && terminal_suffix(&name[end..])
        {
            return std::str::from_utf8(candidate).ok();
        }
    }
    None
}

fn boundary_before(name: &[u8], start: usize) -> bool {
    start == 0 || !name[start - 1].is_ascii_hexdigit()
}

fn terminal_suffix(suffix: &[u8]) -> bool {
    suffix.is_empty() || (suffix.first() == Some(&b'.') && suffix.len() > 1)
}
