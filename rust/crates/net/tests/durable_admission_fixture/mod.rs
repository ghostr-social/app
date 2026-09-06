use ghostr_net::internet_allowance::{InternetAllowance, InternetDataLimit};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use std::path::PathBuf;

pub struct LedgerDirectory(pub(super) PathBuf);

impl LedgerDirectory {
    pub fn open() -> (Self, InternetAllowance) {
        let path = std::env::temp_dir().join(format!("warp-admission-{}", std::process::id()));
        std::fs::create_dir(&path).expect("unique fixture directory");
        let ledger = InternetAllowance::open(&path.join("ledger"), InternetDataLimit::Bytes(100))
            .expect("durable ledger");
        (Self(path), ledger)
    }
}

impl Drop for LedgerDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

pub fn executor(ledger: InternetAllowance) -> MediaRequestExecutor {
    MediaRequestExecutor::with_allowance(
        super::request_gate_fixture::LocalMediaClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("fixture"),
        ledger,
    )
}

pub fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .expect("fixture runtime")
}
