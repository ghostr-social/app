use sha2::{Digest as _, Sha256};

use crate::RequestAuthority;

#[derive(Clone)]
pub struct DecisionPrivacy {
    mode: PrivacyMode,
}

#[derive(Clone)]
enum PrivacyMode {
    Key([u8; 32]),
}

impl DecisionPrivacy {
    pub const fn from_key(key: [u8; 32]) -> Self {
        Self {
            mode: PrivacyMode::Key(key),
        }
    }

    pub(super) fn post(&self, value: &str) -> String {
        self.digest(b"post", value)
    }

    pub(super) fn source(&self, value: &str) -> String {
        let Some(authority) = RequestAuthority::from_url(value) else {
            return self.digest(b"invalid-source", value);
        };
        let authority = self.authority(authority.as_str());
        let source = self.digest(b"source", value);
        format!("{authority}/{source}")
    }

    pub(super) fn authority(&self, value: &str) -> String {
        let Some(authority) = RequestAuthority::from_url(value) else {
            return format!(
                "https://{}.invalid",
                self.digest(b"invalid-authority", value)
            );
        };
        format!(
            "https://{}.invalid",
            self.digest(b"authority", authority.as_str())
        )
    }

    pub(super) fn model_key(&self, value: &str) -> String {
        self.digest(b"model", value)
    }

    fn digest(&self, domain: &[u8], value: &str) -> String {
        let PrivacyMode::Key(key) = self.mode;
        let mut digest = Sha256::new();
        digest.update(key);
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
