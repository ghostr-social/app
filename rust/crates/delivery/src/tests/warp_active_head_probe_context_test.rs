use crate::tests::support::transfer_identity;
use crate::tests::warp_head_probe_context_fixture::{generates_head, plan, state};
use ghostr_engine::adaptive::PlannerCommand;
use ghostr_engine::PostId;

#[test]
fn only_the_current_active_probe_identity_suppresses_head() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    let mut state = state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).unwrap();
    let stale = transfer_identity(&post, source);
    assert!(!generates_head(plan(&mut state, &[current], 2)));
    assert!(generates_head(plan(&mut state, &[stale], 2)));
}

#[test]
fn active_current_head_leaves_one_scoped_body_companion_slot() {
    let post = PostId::new("post");
    let source = "https://media.example/video.mp4";
    let mut state = state(post.clone(), source);
    let current = state.catalog().transfer_identity(&post, source).unwrap();
    let work = plan(&mut state, &[current], 1);
    let selected = work.warp.unwrap().selected.expect("body companion action");

    assert!(matches!(selected.command, PlannerCommand::Transfer(_)));
    assert_eq!(selected.node.post, post);
}
