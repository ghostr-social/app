//! A feed open cannot claim one account while its MainFeed embeds another.

mod support;

use nostr_sdk::Keys;
use rust_lib_ghostr::api::feed_control::{ffi_feed_session, ffi_load_more, ffi_open_feed};
use rust_lib_ghostr::api::session_control::ffi_reset_nostr_session;
use support::feed_session::{main_feed, search_feed};
use support::fixtures::temp_directory;

#[tokio::test]
async fn account_mismatches_do_not_allocate_or_rescope_a_feed() {
    let directory = temp_directory("ghostr-feed-account-guard");
    support::engine::start(&directory, 1024)
        .await
        .expect("engine start");
    let account_a = Keys::generate().public_key().to_hex();
    let account_b = Keys::generate().public_key().to_hex();
    ffi_reset_nostr_session(Some(account_b.clone()))
        .await
        .expect("account B");
    let generation = ffi_feed_session(Some(account_b.clone()))
        .await
        .expect("account B generation");

    assert!(ffi_open_feed(
        main_feed(Some(account_a.clone())),
        Some(account_b.clone()),
        generation,
    )
    .await
    .is_err());
    assert!(
        ffi_open_feed(main_feed(None), Some(account_b.clone()), generation)
            .await
            .is_err()
    );
    let fresh = ffi_open_feed(search_feed(), Some(account_b.clone()), generation)
        .await
        .expect("search feed");
    assert!(ffi_open_feed(main_feed(Some(account_a)), None, generation)
        .await
        .is_err());
    let next = ffi_open_feed(
        main_feed(Some(account_b.clone())),
        Some(account_b),
        generation,
    )
    .await
    .expect("account B main feed");

    assert_eq!(
        next.parse::<u64>().unwrap(),
        fresh.parse::<u64>().unwrap() + 1
    );
    assert!(ffi_load_more(fresh, None)
        .await
        .expect("search remains open"));
    std::fs::remove_dir_all(directory).expect("remove cache");
}
