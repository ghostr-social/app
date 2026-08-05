pub const MAX_NATIVE_IDENTIFIER_BYTES: usize = 512;
pub const MAX_NATIVE_TEXT_CHARACTERS: usize = 4_096;
/// A generous engine-side bound for untrusted URLs. Signed CDN URLs can
/// run to a couple of kilobytes, so the limit should reject only
/// unreasonable input.
pub const MAX_NATIVE_URL_BYTES: usize = 8_192;

pub fn bounded_native_text(value: &str) -> String {
    value.chars().take(MAX_NATIVE_TEXT_CHARACTERS).collect()
}
