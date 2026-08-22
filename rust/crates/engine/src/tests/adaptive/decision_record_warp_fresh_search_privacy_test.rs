use super::fresh_search_support::{planned, record};

#[test]
fn fresh_search_inputs_preserve_authority_groups_without_identifiers() {
    let (state, decision) = planned();
    let json = serde_json::to_string(&record(&state, &decision)).unwrap();

    assert!(json.contains("search_replay_input"));
    assert!(json.contains(".invalid"));
    for secret in ["origin.example", "https://origin", "\"p0\"", "\"p1\""] {
        assert!(
            !json.contains(secret),
            "fresh replay leaked {secret}: {json}"
        );
    }
}
