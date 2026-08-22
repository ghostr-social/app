use ghostr_media_model::blossom::terminal_sha256;

#[test]
fn terminal_hash_uses_the_last_case_insensitive_digest() {
    let older = "1".repeat(64);
    let expected = "A1".repeat(32);
    let url = format!("https://cdn.example/{older}/media/{expected}.MP4");

    assert_eq!(terminal_sha256(&url), Some(expected.to_ascii_lowercase()));
}

#[test]
fn non_terminal_or_malformed_hashes_are_not_blob_identities() {
    let digest = "b".repeat(64);

    assert_eq!(
        terminal_sha256(&format!("https://cdn.example/{digest}/preview.mp4")),
        None
    );
    assert_eq!(
        terminal_sha256(&format!("https://cdn.example/{digest}x.mp4")),
        None
    );
}
