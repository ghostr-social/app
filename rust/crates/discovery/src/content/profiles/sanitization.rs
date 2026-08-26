use nostr_sdk::PublicKey;
use url::Url;

pub(super) fn safe_handle(value: Option<&str>, author: &PublicKey) -> Option<String> {
    let normalized = normalized_text(value)?;
    let without_at = normalized.trim_start_matches('@').trim_start();
    if without_at.eq_ignore_ascii_case(&author.to_hex()) {
        return None;
    }
    bounded_text(without_at, 30)
}

pub(super) fn safe_name(value: Option<&str>, author: &PublicKey, maximum: usize) -> Option<String> {
    let normalized = normalized_text(value)?;
    if normalized.eq_ignore_ascii_case(&author.to_hex()) {
        return None;
    }
    bounded_text(&normalized, maximum)
}

pub(super) fn safe_picture(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.len() > 2048 {
        return None;
    }
    let parsed = Url::parse(raw).ok()?;
    let safe_scheme = matches!(parsed.scheme(), "http" | "https");
    let safe_user = parsed.username().is_empty() && parsed.password().is_none();
    (safe_scheme && safe_user && parsed.host().is_some()).then(|| raw.to_owned())
}

fn bounded_text(value: &str, maximum: usize) -> Option<String> {
    let bounded: String = value.chars().take(maximum).collect();
    (!bounded.is_empty()).then_some(bounded)
}

fn normalized_text(value: Option<&str>) -> Option<String> {
    let mut output = String::new();
    let mut separator = false;
    for character in value?.chars() {
        if unsafe_text(character) {
            separator |= !output.is_empty();
        } else {
            if separator {
                output.push(' ');
                separator = false;
            }
            output.push(character);
        }
    }
    let trimmed = output.trim().to_owned();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn unsafe_text(character: char) -> bool {
    character <= '\u{20}'
        || ('\u{7f}'..='\u{9f}').contains(&character)
        || ('\u{202a}'..='\u{202e}').contains(&character)
        || ('\u{2066}'..='\u{2069}').contains(&character)
}
