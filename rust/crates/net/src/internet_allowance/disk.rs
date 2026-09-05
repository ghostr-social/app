use super::InternetUsage;
use anyhow::{ensure, Context as _, Result};
use fs2::FileExt as _;
use sha2::{Digest as _, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"WARPDA01";
const RECORD_BYTES: usize = 56;

pub(super) struct LedgerDisk {
    path: PathBuf,
    _owner: File,
}

impl LedgerDisk {
    pub(super) fn open(path: &Path) -> Result<Self> {
        let parent = path.parent().context("Internet ledger needs a directory")?;
        std::fs::create_dir_all(parent)?;
        let owner = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))?;
        owner
            .try_lock_exclusive()
            .context("Internet ledger already has an owner")?;
        Ok(Self {
            path: path.to_owned(),
            _owner: owner,
        })
    }

    pub(super) fn load(&self) -> Result<InternetUsage> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(InternetUsage::default())
            }
            Err(error) => return Err(error.into()),
        };
        let mut bytes = Vec::with_capacity(RECORD_BYTES + 1);
        file.take((RECORD_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        decode(&bytes)
    }

    pub(super) fn save(&self, usage: InternetUsage) -> Result<()> {
        let pending = self.path.with_extension("pending");
        let mut file = File::create(&pending)?;
        file.write_all(&encode(usage))?;
        file.sync_all()?;
        std::fs::rename(&pending, &self.path)?;
        File::open(
            self.path
                .parent()
                .context("Internet ledger directory missing")?,
        )?
        .sync_all()?;
        Ok(())
    }
}

fn encode(usage: InternetUsage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(RECORD_BYTES);
    bytes.extend(MAGIC);
    bytes.extend(usage.charged_bytes.to_be_bytes());
    bytes.extend(usage.reserved_bytes.to_be_bytes());
    let checksum = Sha256::digest(&bytes);
    bytes.extend(checksum);
    bytes
}

fn decode(bytes: &[u8]) -> Result<InternetUsage> {
    ensure!(
        bytes.len() == RECORD_BYTES && &bytes[..8] == MAGIC,
        "invalid Internet ledger"
    );
    ensure!(
        Sha256::digest(&bytes[..24])[..] == bytes[24..],
        "corrupt Internet ledger"
    );
    Ok(InternetUsage {
        charged_bytes: u64::from_be_bytes(bytes[8..16].try_into()?),
        reserved_bytes: u64::from_be_bytes(bytes[16..24].try_into()?),
    })
}
