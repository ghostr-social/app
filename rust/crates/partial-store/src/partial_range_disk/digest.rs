use anyhow::{Context as _, Result};
use core::ops::Range;
use sha2::{Digest as _, Sha256};
use std::io::SeekFrom;
use std::path::Path;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};

const BUFFER_BYTES: usize = 64 * 1024;

pub(super) fn bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) async fn file(path: &Path) -> Result<String> {
    let length = tokio::fs::metadata(path)
        .await
        .context("inspect partial video for digest")?
        .len();
    span(path, &(0..length)).await
}

pub(super) async fn span(path: &Path, span: &Range<u64>) -> Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context("open partial video for digest")?;
    file.seek(SeekFrom::Start(span.start))
        .await
        .context("seek partial video for digest")?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut remaining = span.end.saturating_sub(span.start);
    while remaining > 0 {
        let wanted = remaining.min(BUFFER_BYTES as u64) as usize;
        file.read_exact(&mut buffer[..wanted])
            .await
            .context("read partial video for digest")?;
        digest.update(&buffer[..wanted]);
        remaining -= wanted as u64;
    }
    Ok(format!("{:x}", digest.finalize()))
}
