use core::fmt::Write as _;
use sha2::{Digest as _, Sha256};

pub(super) struct EvaluationPrivacy([u8; 32]);

impl Default for EvaluationPrivacy {
    fn default() -> Self {
        Self(rand::random())
    }
}

impl EvaluationPrivacy {
    pub(super) fn origin(&self, value: &str) -> String {
        let mut digest = Sha256::new();
        digest.update(self.0);
        digest.update(b"origin");
        digest.update(value.as_bytes());
        let mut pseudonym = String::with_capacity(24);
        for byte in &digest.finalize()[..12] {
            write!(pseudonym, "{byte:02x}").expect("writing to a String is infallible");
        }
        pseudonym
    }
}
