use rust_lib_ghostr::video::progressive_posts::ServablePosts;

#[test]
fn removing_a_progressive_post_revokes_gateway_access() {
    let posts = ServablePosts::new();
    posts.insert("clip");
    assert!(posts.contains("clip"));

    posts.remove("clip");

    assert!(!posts.contains("clip"));
}
