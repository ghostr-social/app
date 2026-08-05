//! A handle whose owning runtime ended reports a stopped scheduler.

use super::scheduler_support::start_scheduler;
use crate::engine::DataUsageLevel;
use tokio::runtime::Builder;

#[test]
fn reset_after_scheduler_shutdown_reports_stopped() {
    let owner = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("owner runtime");
    let handle =
        owner.block_on(async { start_scheduler(DataUsageLevel::Conservative, Vec::new()).handle });
    drop(owner);
    let caller = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("caller runtime");

    let failure = caller
        .block_on(handle.reset_session())
        .expect_err("the scheduler task ended with its runtime");

    assert_eq!(failure.message, "the discovery scheduler stopped");
}
