use super::LedgerDirectory;
use std::os::unix::fs::PermissionsExt as _;

pub struct ReadOnly<'a>(&'a LedgerDirectory);

impl LedgerDirectory {
    pub fn read_only(&self) -> ReadOnly<'_> {
        std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o500))
            .expect("read-only ledger directory");
        ReadOnly(self)
    }
}

impl Drop for ReadOnly<'_> {
    fn drop(&mut self) {
        std::fs::set_permissions(&self.0 .0, std::fs::Permissions::from_mode(0o700))
            .expect("restore fixture directory");
    }
}
