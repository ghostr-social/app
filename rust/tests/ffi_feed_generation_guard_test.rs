//! A reset rejects work captured by an earlier native session generation.

mod support;

use nostr_sdk::Keys;
use rust_lib_ghostr::api::feed_control::{ffi_feed_session, ffi_open_feed};
use rust_lib_ghostr::api::session_control::ffi_reset_nostr_session;
use support::feed_session::{main_feed, search_feed};
use support::fixtures::temp_directory;

#[tokio::test]
async fn same_account_and_sign_out_resets_reject_stale_opens() {
    let directory = temp_directory("ghostr-feed-generation-guard");
    support::engine::start(&directory, 1024)
        .await
        .expect("engine start");
    let account = Keys::generate().public_key().to_hex();
    ffi_reset_nostr_session(Some(account.clone()))
        .await
        .expect("account session");
    let stale_generation = ffi_feed_session(Some(account.clone()))
        .await
        .expect("old generation");

    ffi_reset_nostr_session(Some(account.clone()))
        .await
        .expect("same-account reset");
    assert!(
        ffi_open_feed(search_feed(), Some(account.clone()), stale_generation,)
            .await
            .is_err()
    );
    let fresh_generation = ffi_feed_session(Some(account.clone()))
        .await
        .expect("fresh generation");
    let fresh = ffi_open_feed(search_feed(), Some(account.clone()), fresh_generation)
        .await
        .expect("fresh feed");

    ffi_reset_nostr_session(None).await.expect("sign out");
    assert!(
        ffi_open_feed(search_feed(), Some(account), fresh_generation)
            .await
            .is_err()
    );
    let signed_out = ffi_feed_session(None).await.expect("signed-out generation");
    let next = ffi_open_feed(main_feed(None), None, signed_out)
        .await
        .expect("signed-out feed");
    assert_eq!(
        next.parse::<u64>().unwrap(),
        fresh.parse::<u64>().unwrap() + 1
    );
    std::fs::remove_dir_all(directory).expect("remove cache");
}
