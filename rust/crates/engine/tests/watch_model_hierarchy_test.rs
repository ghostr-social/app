use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchSample, WatchSampleKind};

fn context(video: &str, creator: &str, category: &str, duration: u64) -> WatchContext {
    WatchContext::new(WatchKey::digest(video), Some(duration))
        .with_creator(WatchKey::digest(creator))
        .with_categories([WatchKey::digest(category)])
        .with_user(WatchKey::digest("local-user"))
}

#[test]
fn cold_videos_shrink_to_creator_category_duration_user_and_global_priors() {
    let mut model = WatchModel::default();
    for index in 0..12 {
        model.observe(&WatchSample::new(
            context(&format!("trained-{index}"), "creator-a", "music", 20_000),
            18_000,
            WatchSampleKind::Completed,
            1_000 + index,
        ));
    }
    let related = context("cold-related", "creator-a", "music", 20_000);
    let unrelated = context("cold-other", "creator-b", "sports", 60_000);

    assert!(
        model.predict(&related, 20_000).survival(12_000)
            > model.predict(&unrelated, 20_000).survival(12_000)
    );
    assert!(model.predict(&unrelated, 20_000).p50_ms() > 0);
}

#[test]
fn enough_video_specific_evidence_can_override_a_creator_prior() {
    let mut model = WatchModel::default();
    for index in 0..10 {
        model.observe(&WatchSample::new(
            context(&format!("long-{index}"), "creator", "topic", 20_000),
            18_000,
            WatchSampleKind::Completed,
            1_000 + index,
        ));
    }
    let target = context("target", "creator", "topic", 20_000);
    let inherited = model.predict(&target, 2_000).p50_ms();
    for index in 0..12 {
        model.observe(&WatchSample::new(
            target.clone(),
            800,
            WatchSampleKind::Abandoned,
            3_000 + index,
        ));
    }

    assert!(model.predict(&target, 5_000).p50_ms() < inherited);
}
