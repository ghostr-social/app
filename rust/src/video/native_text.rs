pub const MAX_NATIVE_IDENTIFIER_BYTES: usize = 512;
pub const MAX_NATIVE_TEXT_CHARACTERS: usize = 4_096;
/// Rust-side hardening the Dart pipeline has no equivalent of, so it
/// sits far past any real link: signed CDN URLs run to a couple of
/// kilobytes, and a bound that rejects one costs the feed a post it
/// would otherwise play.
pub const MAX_NATIVE_URL_BYTES: usize = 8_192;

pub fn bounded_native_text(value: &str) -> String {
    value.chars().take(MAX_NATIVE_TEXT_CHARACTERS).collect()
}
