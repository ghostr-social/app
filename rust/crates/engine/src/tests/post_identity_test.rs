use crate::{ByteRange, ChunkId, PostId};

#[test]
fn post_id_wraps_the_raw_event_id() {
    let post = PostId::new("abc123");

    assert_eq!(post.as_str(), "abc123");
    assert_eq!(post, PostId::new(String::from("abc123")));
    assert_ne!(post, PostId::new("other"));
}

#[test]
fn chunk_identity_is_post_plus_range() {
    let chunk = ChunkId {
        post: PostId::new("abc"),
        range: ByteRange::new(0, 10),
    };
    let same = ChunkId {
        post: PostId::new("abc"),
        range: ByteRange::new(0, 10),
    };
    let other_range = ChunkId {
        post: PostId::new("abc"),
        range: ByteRange::new(10, 20),
    };

    assert_eq!(chunk, same);
    assert_ne!(chunk, other_range);
}
