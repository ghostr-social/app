use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

mod tail;
pub(super) use tail::TailIntent;

const POLICY_INTENT_VERSION: u8 = 2;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransactionIntent {
    version: u8,
    old_accounted: u64,
    new_accounted: u64,
    old_manifest_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyIntent {
    version: u8,
    retained_bytes: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DiskIntent {
    Tail(TailIntent),
    Transaction(TransactionIntent),
    Legacy(LegacyIntent),
}

pub(super) enum PolicyIntent {
    Tail(TailIntent),
    Transaction(TransactionIntent),
    Legacy { retained_bytes: u64 },
}

impl TransactionIntent {
    pub(super) fn new(old: u64, new: u64, old_manifest_sha256: String) -> Self {
        Self {
            version: POLICY_INTENT_VERSION,
            old_accounted: old,
            new_accounted: new,
            old_manifest_sha256,
        }
    }

    pub(super) fn old_accounted(&self) -> u64 {
        self.old_accounted
    }

    pub(super) fn new_accounted(&self) -> u64 {
        self.new_accounted
    }

    pub(super) fn old_manifest_sha256(&self) -> &str {
        &self.old_manifest_sha256
    }
}

pub(super) async fn load(paths: &StorePaths, key: &str) -> Result<Option<PolicyIntent>> {
    let text = match tokio::fs::read_to_string(paths.policy_intent(key)).await {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read policy eviction intent"),
    };
    match serde_json::from_str(&text).context("parse policy eviction intent")? {
        DiskIntent::Tail(intent) => {
            intent.validate()?;
            Ok(Some(PolicyIntent::Tail(intent)))
        }
        DiskIntent::Transaction(intent) => {
            validate(&intent)?;
            Ok(Some(PolicyIntent::Transaction(intent)))
        }
        DiskIntent::Legacy(intent) => {
            ensure!(intent.version == 1, "unsupported policy eviction intent");
            Ok(Some(PolicyIntent::Legacy {
                retained_bytes: intent.retained_bytes,
            }))
        }
    }
}

pub(super) async fn save(paths: &StorePaths, key: &str, intent: &TransactionIntent) -> Result<()> {
    validate(intent)?;
    save_value(paths, key, intent).await
}

pub(super) async fn save_tail(paths: &StorePaths, key: &str, intent: &TailIntent) -> Result<()> {
    intent.validate()?;
    save_value(paths, key, intent).await
}

async fn save_value(paths: &StorePaths, key: &str, intent: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec(intent).context("encode policy eviction intent")?;
    disk::save_durable(
        &paths.policy_intent(key),
        &paths.policy_intent_staging(key),
        &bytes,
    )
    .await
}

pub(super) async fn exists(paths: &StorePaths, key: &str) -> Result<bool> {
    match tokio::fs::symlink_metadata(paths.policy_intent(key)).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).context("inspect policy eviction intent"),
    }
}

pub(super) async fn remove_authority(paths: &StorePaths, key: &str) -> Result<()> {
    let present = exists(paths, key).await?;
    disk::remove_if_present(&paths.policy_intent_staging(key)).await?;
    if present {
        disk::remove_durable(&paths.policy_intent(key)).await?;
    }
    Ok(())
}

pub(super) async fn payload_bytes(paths: &StorePaths, key: &str) -> u64 {
    let present = disk::file_len(&paths.policy_staging(key))
        .await
        .ok()
        .flatten();
    let Some(length) = present else { return 0 };
    match load(paths, key).await {
        Ok(Some(PolicyIntent::Tail(_))) => 0,
        Ok(Some(PolicyIntent::Transaction(intent))) => intent.new_accounted(),
        Ok(Some(PolicyIntent::Legacy { retained_bytes })) if retained_bytes <= length => {
            retained_bytes
        }
        _ => length,
    }
}

fn validate(intent: &TransactionIntent) -> Result<()> {
    ensure!(
        intent.version == POLICY_INTENT_VERSION,
        "unsupported policy intent"
    );
    ensure!(intent.old_accounted > 0, "policy intent has no old bytes");
    ensure!(
        intent.new_accounted > 0,
        "policy intent has no retained bytes"
    );
    ensure!(
        intent.new_accounted < intent.old_accounted,
        "policy intent does not evict bytes"
    );
    ensure!(
        valid_digest(&intent.old_manifest_sha256),
        "invalid old manifest hash"
    );
    Ok(())
}

pub(super) fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
