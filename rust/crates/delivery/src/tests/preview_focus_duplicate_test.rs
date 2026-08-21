use super::preview_focus_fixture::{candidate_state, focus};
use crate::delivery_events::FocusPreview;
use ghostr_engine::PreviewDescriptor;

#[test]
fn duplicate_focus_previews_resolve_to_the_first_valid_descriptor() {
    let (mut state, post, meta) = candidate_state();
    let first = PreviewDescriptor::inline_blurhash("000000").unwrap();
    let second = PreviewDescriptor::inline_blurhash("LEHV6nWB2yk8pyo0adR*.7kCMdnj").unwrap();
    let mut update = focus(post.clone(), meta);
    update.previews = vec![
        FocusPreview {
            post: post.clone(),
            descriptor: first,
        },
        FocusPreview {
            post: post.clone(),
            descriptor: second,
        },
    ];

    assert!(state.apply_focus(update, 2));
    assert_eq!(
        state.catalog().lookup(&post).unwrap().preview(),
        Some(first)
    );
}
