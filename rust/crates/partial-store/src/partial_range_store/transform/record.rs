use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context, Result};
use ghostr_engine::adaptive::TransformKind;
use ghostr_engine::representation::RepresentationBinding;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransformRecord {
    version: u8,
    input: String,
    output: String,
    kind: String,
    digest: String,
    bytes: u64,
}

impl TransformRecord {
    pub(super) fn new(
        input: &RepresentationBinding,
        output: &RepresentationBinding,
        kind: TransformKind,
        digest: String,
        bytes: u64,
    ) -> Self {
        Self {
            version: 1,
            input: input.representation().fingerprint().to_owned(),
            output: output.representation().fingerprint().to_owned(),
            kind: encode_kind(kind).to_owned(),
            digest,
            bytes,
        }
    }

    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }
    pub(super) fn digest(&self) -> &str {
        &self.digest
    }
    pub(super) fn output(&self) -> &str {
        &self.output
    }

    fn kind(&self) -> Result<TransformKind> {
        match self.kind.as_str() {
            "remux" => Ok(TransformKind::Remux),
            "segment" => Ok(TransformKind::Segment),
            "transcode" => Ok(TransformKind::Transcode),
            _ => anyhow::bail!("stored transform kind is invalid"),
        }
    }
}

pub(super) async fn save_staging(
    paths: &StorePaths,
    key: &str,
    record: &TransformRecord,
) -> Result<()> {
    let path = paths.transform(key).record_staging();
    let staging = path.with_extension("json.tmp");
    let json = serde_json::to_vec(record).context("encode transform provenance")?;
    disk::save_durable(&path, &staging, &json).await
}

pub(super) async fn load(paths: &StorePaths, key: &str) -> Result<Option<TransformRecord>> {
    let bytes = match tokio::fs::read(paths.transform(key).record()).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read transform provenance"),
    };
    let record: TransformRecord =
        serde_json::from_slice(&bytes).context("decode transform provenance")?;
    ensure!(
        record.version == 1 && record.bytes > 0,
        "invalid transform provenance version or size"
    );
    Ok(Some(record))
}

pub(super) async fn restore_binding(
    paths: &StorePaths,
    input: &RepresentationBinding,
    stored: Option<&str>,
) -> Result<Option<RepresentationBinding>> {
    let Some(record) = load(paths, input.post().as_str()).await? else {
        return Ok(None);
    };
    if record.input != input.representation().fingerprint() || Some(record.output()) != stored {
        return Ok(None);
    }
    let derived = input.derive_transform(record.kind()?, record.digest());
    Ok(derived.filter(|binding| binding.representation().fingerprint() == record.output()))
}

fn encode_kind(kind: TransformKind) -> &'static str {
    match kind {
        TransformKind::Remux => "remux",
        TransformKind::Segment => "segment",
        TransformKind::Transcode => "transcode",
    }
}
