use rust_lib_ghostr::api::feed_types::{FfiFeedKind, FfiFeedSpec};

pub fn main_feed(viewer_pubkey: Option<String>) -> FfiFeedSpec {
    FfiFeedSpec {
        kind: FfiFeedKind::Main,
        value: None,
        creators: Vec::new(),
        viewer_pubkey,
    }
}

pub fn search_feed() -> FfiFeedSpec {
    FfiFeedSpec {
        kind: FfiFeedKind::Search,
        value: Some("ghost".to_owned()),
        creators: Vec::new(),
        viewer_pubkey: None,
    }
}
