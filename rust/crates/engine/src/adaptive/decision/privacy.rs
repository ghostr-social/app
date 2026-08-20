use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct DecisionPrivacy {
    key: [u8; 32],
}

impl DecisionPrivacy {
    pub const fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub(super) fn post(&self, value: &str) -> String {
        self.digest(b"post", value)
    }

    pub(super) fn source(&self, value: &str) -> String {
        self.digest(b"source", value)
    }

    fn digest(&self, domain: &[u8], value: &str) -> String {
        let mut digest = Sha256::new();
        digest.update(self.key);
        digest.update(domain);
        digest.update(value.as_bytes());
        hex(&digest.finalize()[..16])
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}
