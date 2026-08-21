use super::preview_focus_fixture::{candidate_state, focus, meta, BLURHASH};
use ghostr_engine::PreviewDescriptor;

#[test]
fn omitted_focus_preview_retains_same_binding_but_not_a_replacement() {
    let (mut state, post, original) = candidate_state();

    assert!(state.apply_focus(focus(post.clone(), original), 2));
    assert_eq!(
        state.catalog().lookup(&post).unwrap().preview(),
        PreviewDescriptor::inline_blurhash(BLURHASH)
    );

    let replacement = meta("https://media.example/replacement.mp4");
    assert!(state.apply_focus(focus(post.clone(), replacement), 3));
    assert_eq!(state.catalog().lookup(&post).unwrap().preview(), None);
}
