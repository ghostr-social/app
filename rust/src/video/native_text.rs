pub const MAX_NATIVE_IDENTIFIER_BYTES: usize = 512;
pub const MAX_NATIVE_TEXT_CHARACTERS: usize = 4_096;
pub const MAX_NATIVE_URL_BYTES: usize = 2_048;

pub fn bounded_native_text(value: &str) -> String {
    value.chars().take(MAX_NATIVE_TEXT_CHARACTERS).collect()
}
