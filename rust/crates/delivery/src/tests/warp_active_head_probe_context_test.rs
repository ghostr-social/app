use crate::tests::support::transfer_identity;
use crate::tests::warp_head_probe_context_fixture::{
    ahead_state, current_state, generates_head_for, plan,
};
use ghostr_engine::adaptive::PlannerCommand;
use ghostr_engine::PostId;

#[test]
fn only_the_matching_ahead_probe_identity_suppresses_head() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    let state = ahead_state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).expect("valid test fixture");
    let stale = transfer_identity(&post, source);
    assert!(!generates_head_for(plan(&state, &[current], 2), &post));
    assert!(generates_head_for(plan(&state, &[stale], 2), &post));
}

#[test]
fn inherited_current_head_leaves_one_scoped_body_companion_slot() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    // A probe launched while this post was ahead can remain active after a swipe.
    let state = current_state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).expect("valid test fixture");
    let work = plan(&state, &[current], 1);
    let selected = work.warp.expect("valid test fixture").selected.expect("body companion action");

    assert!(matches!(selected.command, PlannerCommand::Transfer(_)));
    assert_eq!(selected.node.post, post);
}
