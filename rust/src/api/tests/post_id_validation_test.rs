use crate::api::focus_mapping::validate_post_id;

#[test]
fn accepts_store_safe_post_ids() {
    validate_post_id("abc-DEF_123").expect("store-safe id");
}

#[test]
fn rejects_post_ids_that_cannot_be_store_keys() {
    for id in ["", "a/b", "a b", "a?b=1", "sp\u{00e9}cial"] {
        assert!(validate_post_id(id).is_err(), "id {id:?} must be rejected");
    }
}
