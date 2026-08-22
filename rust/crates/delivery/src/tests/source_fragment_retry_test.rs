use crate::manager::retry::{RetryBook, RetryPolicy};
use ghostr_engine::PostId;

#[test]
fn fragment_alias_is_not_an_independent_retry_alternative() {
    let post = PostId::new("clip");
    let failed = "https://origin.example/clip.mp4#old";
    let aliases = vec!["https://origin.example/clip.mp4#new".to_owned()];
    let retry = RetryBook::new(RetryPolicy::default());

    assert!(!retry.has_ready_alternative(&post, failed, &aliases));
}
