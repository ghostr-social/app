use crate::api::debug::nostr::{
    debug_item, debug_stage, environment_relays, DebugNostrConfiguration,
};
use crate::api::feed::mapping::feed_post;
use crate::api::feed_types::FfiFeedStage;
use crate::api::tests::support::{creator_profile, parsed_video_post};
use crate::discovery::content::profiles::CreatorProfile;
use crate::engine::DeliveryKind;
use ghostr_delivery::debug::feed::DebugFeedStage;

#[test]
fn progressive_feed_rows_become_debug_delivery_items() {
    let post = parsed_video_post(34_235, Some("clip"));
    let row = feed_post(&post, creator_profile(), None);

    let item = debug_item(row).expect("progressive video");

    assert_eq!(item.event_id, "e1");
    assert_eq!(item.title.as_deref(), Some("sunset ride"));
    assert_eq!(item.creator, "Vera");
    assert_eq!(item.meta.delivery, DeliveryKind::Progressive);
}

#[test]
fn hls_rows_stay_in_the_shared_debug_feed() {
    let mut post = parsed_video_post(34_235, Some("clip"));
    post.meta.delivery = DeliveryKind::Hls;
    let row = feed_post(&post, creator_profile(), None);

    let item = debug_item(row).expect("HLS video");

    assert_eq!(item.meta.delivery, DeliveryKind::Hls);
}

#[test]
fn explicit_titles_and_profile_handles_are_preserved() {
    let mut post = parsed_video_post(34_235, Some("clip"));
    post.title = Some("Native title".to_owned());
    let profile = CreatorProfile {
        display_name: " ".to_owned(),
        handle: "@fallback".to_owned(),
        avatar_url: None,
    };
    let item = debug_item(feed_post(&post, profile, None)).expect("progressive video");

    assert_eq!(item.title.as_deref(), Some("Native title"));
    assert_eq!(item.creator, "@fallback");
}

#[test]
fn stages_and_relay_overrides_keep_the_native_vocabulary() {
    assert_eq!(debug_stage(FfiFeedStage::Loading), DebugFeedStage::Loading);
    assert_eq!(debug_stage(FfiFeedStage::Settled), DebugFeedStage::Settled);
    assert_eq!(debug_stage(FfiFeedStage::Failed), DebugFeedStage::Failed);

    let key = "GHOSTR_TEST_RELAYS_NOT_SET";
    assert_eq!(
        environment_relays(key, vec!["wss://fallback.example".to_owned()]).expect("fallback"),
        ["wss://fallback.example"]
    );
}

#[test]
fn environment_configuration_uses_both_relay_overrides() {
    let read = std::env::var_os("GHOSTR_NOSTR_RELAYS");
    let search = std::env::var_os("GHOSTR_NOSTR_SEARCH_RELAYS");
    std::env::set_var("GHOSTR_NOSTR_RELAYS", "wss://read.example");
    std::env::set_var("GHOSTR_NOSTR_SEARCH_RELAYS", "wss://search.example");

    let configuration = DebugNostrConfiguration::from_environment().expect("configuration");

    restore_environment("GHOSTR_NOSTR_RELAYS", read);
    restore_environment("GHOSTR_NOSTR_SEARCH_RELAYS", search);
    assert_eq!(configuration.read_relays, ["wss://read.example"]);
    assert_eq!(configuration.search_relays, ["wss://search.example"]);
}

fn restore_environment(key: &str, value: Option<std::ffi::OsString>) {
    if let Some(value) = value {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
}
