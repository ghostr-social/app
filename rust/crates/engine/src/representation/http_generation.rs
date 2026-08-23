use crate::evidence::EvidenceValidator;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::NonZeroU64;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct HttpGenerationKey {
    final_url: String,
    validator: Option<EvidenceValidator>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HttpGenerationEpoch(NonZeroU64);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HttpGenerationLease {
    key: HttpGenerationKey,
    epoch: HttpGenerationEpoch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpGenerationStamp {
    key: HttpGenerationKey,
    authority: HttpGenerationAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpGenerationAuthority {
    Trusted(HttpGenerationLease),
    Unknown(HttpGenerationEpoch),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidHttpGeneration;

impl HttpGenerationKey {
    pub fn try_new(
        final_url: impl Into<String>,
        validator: Option<EvidenceValidator>,
    ) -> Result<Self, InvalidHttpGeneration> {
        let final_url = final_url.into();
        if final_url.is_empty() || final_url.bytes().any(|byte| byte.is_ascii_control()) {
            return Err(InvalidHttpGeneration);
        }
        Ok(Self {
            final_url,
            validator,
        })
    }

    pub fn final_url(&self) -> &str {
        &self.final_url
    }

    pub const fn validator(&self) -> Option<&EvidenceValidator> {
        self.validator.as_ref()
    }
}

impl HttpGenerationEpoch {
    pub fn try_new(value: u64) -> Result<Self, InvalidHttpGeneration> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(InvalidHttpGeneration)
    }

    pub const fn value(self) -> u64 {
        self.0.get()
    }
}

impl HttpGenerationLease {
    pub fn try_new(key: HttpGenerationKey, epoch: u64) -> Result<Self, InvalidHttpGeneration> {
        Ok(Self {
            key,
            epoch: HttpGenerationEpoch::try_new(epoch)?,
        })
    }

    pub const fn key(&self) -> &HttpGenerationKey {
        &self.key
    }

    pub const fn epoch(&self) -> HttpGenerationEpoch {
        self.epoch
    }
}

impl HttpGenerationStamp {
    pub fn from_trusted(lease: HttpGenerationLease) -> Self {
        Self {
            key: lease.key().clone(),
            authority: HttpGenerationAuthority::Trusted(lease),
        }
    }

    pub(crate) fn new(key: HttpGenerationKey, authority: HttpGenerationAuthority) -> Self {
        let coherent = match &authority {
            HttpGenerationAuthority::Trusted(lease) => lease.key() == &key,
            HttpGenerationAuthority::Unknown(_) => true,
        };
        debug_assert!(coherent);
        Self { key, authority }
    }

    pub const fn key(&self) -> &HttpGenerationKey {
        &self.key
    }

    pub const fn authority(&self) -> &HttpGenerationAuthority {
        &self.authority
    }
}

impl HttpGenerationAuthority {
    pub const fn epoch(&self) -> HttpGenerationEpoch {
        match self {
            Self::Trusted(lease) => lease.epoch(),
            Self::Unknown(epoch) => *epoch,
        }
    }
}

impl Display for InvalidHttpGeneration {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("HTTP generation requires a final URL and nonzero epoch")
    }
}

impl std::error::Error for InvalidHttpGeneration {}
