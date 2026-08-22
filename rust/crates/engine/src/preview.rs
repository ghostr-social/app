const BASE83: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz#$%*+,-.:;=?@[]^_{|}~";

/// Validated, privacy-minimized evidence that a preview is already inline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewDescriptor {
    InlineBlurhash { encoded_bytes: u16 },
}

impl PreviewDescriptor {
    pub fn inline_blurhash(value: &str) -> Option<Self> {
        valid_blurhash(value).then_some(Self::InlineBlurhash {
            encoded_bytes: value.len() as u16,
        })
    }

    pub const fn encoded_bytes(self) -> u64 {
        match self {
            Self::InlineBlurhash { encoded_bytes } => encoded_bytes as u64,
        }
    }
}

fn valid_blurhash(value: &str) -> bool {
    let bytes = value.as_bytes();
    let Some(components) = bytes.first().and_then(base83_index) else {
        return false;
    };
    if components > 80 {
        return false;
    }
    let width = components % 9 + 1;
    let height = components / 9 + 1;
    bytes.len() == 4 + 2 * width * height && bytes.iter().all(|byte| base83_index(byte).is_some())
}

fn base83_index(byte: &u8) -> Option<usize> {
    BASE83.iter().position(|candidate| candidate == byte)
}
