use crate::catalog::{Catalog, CatalogEntry};
use crate::representation::{HttpGenerationAuthority, HttpGenerationStamp};
use crate::PostId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::catalog) struct VerifiedMirrorRecord {
    digest: String,
    generation: HttpGenerationStamp,
}

impl CatalogEntry {
    pub(super) fn record_verified_mirror(
        &mut self,
        source: &str,
        digest: &str,
        generation: HttpGenerationStamp,
    ) -> bool {
        let current = self.http_generation_stamp(source);
        if current.as_ref() != Some(&generation) || !trusted_strong(&generation) {
            return false;
        }
        self.verified_mirrors.insert(
            source.to_owned(),
            VerifiedMirrorRecord {
                digest: digest.to_ascii_lowercase(),
                generation,
            },
        );
        true
    }

    fn current_verified_mirror(&self, source: &str) -> Option<&VerifiedMirrorRecord> {
        let record = self.verified_mirrors.get(source)?;
        let current = self.http_generation_stamp(source)?;
        (current == record.generation && trusted_strong(&current)).then_some(record)
    }
}

impl Catalog {
    pub fn verified_mirror_digest<'a>(
        &'a self,
        post: &PostId,
        primary: &str,
        alternate: &str,
    ) -> Option<&'a str> {
        let entry = self.entries.get(post)?;
        if entry.quarantined {
            return None;
        }
        let primary = entry.current_verified_mirror(primary)?;
        let alternate = entry.current_verified_mirror(alternate)?;
        primary
            .digest
            .eq_ignore_ascii_case(&alternate.digest)
            .then_some(primary.digest.as_str())
    }
}

fn trusted_strong(stamp: &HttpGenerationStamp) -> bool {
    let HttpGenerationAuthority::Trusted(lease) = stamp.authority() else {
        return false;
    };
    lease
        .key()
        .validator()
        .is_some_and(|value| value.is_strong())
}
