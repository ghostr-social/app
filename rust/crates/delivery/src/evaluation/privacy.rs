use sha2::{Digest, Sha256};

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
        digest.finalize()[..12]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
