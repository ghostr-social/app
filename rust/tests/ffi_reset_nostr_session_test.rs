//! A started engine survives reset while every old feed ends.

mod support;

use rust_lib_ghostr::api::feed_control::{ffi_feed_session, ffi_load_more, ffi_open_feed};
use rust_lib_ghostr::api::session_control::ffi_reset_nostr_session;
use support::feed_session::main_feed;
use support::fixtures::temp_directory;

#[tokio::test]
async fn reset_closes_feeds_without_reusing_their_handles() {
    let directory = temp_directory("ghostr-nostr-session-reset");
    support::engine::start(&directory, 1024)
        .await
        .expect("engine start");
    let stale_generation = ffi_feed_session(None).await.expect("old session");
    let stale = ffi_open_feed(main_feed(None), None, stale_generation)
        .await
        .expect("old feed");
    assert!(ffi_load_more(stale.clone(), None).await.expect("open feed"));

    ffi_reset_nostr_session(None).await.expect("session reset");

    assert!(!ffi_load_more(stale.clone(), None)
        .await
        .expect("stale feed"));
    let fresh_generation = ffi_feed_session(None).await.expect("fresh session");
    let fresh = ffi_open_feed(main_feed(None), None, fresh_generation)
        .await
        .expect("new feed");
    assert!(
        fresh.parse::<u64>().expect("fresh feed id must be numeric")
            > stale.parse::<u64>().expect("stale feed id must be numeric")
    );
    std::fs::remove_dir_all(directory).expect("remove cache");
}
