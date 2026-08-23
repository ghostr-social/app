use super::Catalog;
use crate::evidence::FieldReliabilityModel;
use crate::PostId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CatalogEvidenceState {
    reliability: FieldReliabilityModel,
    digest_claims: BTreeMap<String, BTreeSet<String>>,
    quarantined_digests: BTreeSet<String>,
}

impl CatalogEvidenceState {
    pub fn from_reliability(reliability: FieldReliabilityModel) -> Self {
        Self {
            reliability,
            ..Self::default()
        }
    }

    pub fn reliability(&self) -> &FieldReliabilityModel {
        &self.reliability
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("catalog evidence always serializes")
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut state: Self = serde_json::from_str(json)?;
        state.reliability = FieldReliabilityModel::normalized(state.reliability);
        Ok(state)
    }
}

impl Catalog {
    pub fn evidence_state(&self) -> CatalogEvidenceState {
        CatalogEvidenceState {
            reliability: self.reliability.clone(),
            digest_claims: encode_claims(&self.persisted_digest_claims()),
            quarantined_digests: self.quarantined_digests.clone(),
        }
    }

    pub fn replace_evidence_state(&mut self, state: CatalogEvidenceState, now_ms: u64) {
        self.reliability = state.reliability;
        self.digest_claims = decode_claims(state.digest_claims);
        self.quarantined_digests = state.quarantined_digests;
        self.recalibrate(now_ms);
        let posts: Vec<_> = self.entries.keys().cloned().collect();
        for post in posts {
            self.apply_known_quarantine(&post);
        }
    }

    fn persisted_digest_claims(&self) -> HashMap<String, BTreeSet<PostId>> {
        let mut claims = self.digest_claims.clone();
        claims.retain(|_, posts| {
            posts.retain(|post| !self.entries.contains_key(post));
            !posts.is_empty()
        });
        for (post, entry) in &self.entries {
            let Some(digest) = entry.renditions.advertised_digest() else {
                continue;
            };
            claims
                .entry(digest.to_ascii_lowercase())
                .or_default()
                .insert(post.clone());
        }
        claims
    }
}

fn encode_claims(claims: &HashMap<String, BTreeSet<PostId>>) -> BTreeMap<String, BTreeSet<String>> {
    claims
        .iter()
        .map(|(digest, posts)| {
            let posts = posts.iter().map(|post| post.as_str().to_owned()).collect();
            (digest.clone(), posts)
        })
        .collect()
}

fn decode_claims(claims: BTreeMap<String, BTreeSet<String>>) -> HashMap<String, BTreeSet<PostId>> {
    claims
        .into_iter()
        .map(|(digest, posts)| {
            let posts = posts.into_iter().map(PostId::new).collect();
            (digest.to_ascii_lowercase(), posts)
        })
        .collect()
}
