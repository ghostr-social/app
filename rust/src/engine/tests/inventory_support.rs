use crate::engine::catalog::Catalog;
use crate::engine::focus::{FocusState, FocusUpdate};
use crate::engine::inventory_controller::PresentRanges;
use crate::engine::tests::support::progressive_meta;
use crate::engine::{ByteRange, PostId};

pub struct Scenario {
    pub catalog: Catalog,
    pub focus: FocusState,
    pub present: PresentRanges,
    pub posts: Vec<PostId>,
}

/// A window of `window` tiny posts (1000 bytes, 1 s → the head is the
/// whole file), the first `startable` of them fully on disk, and the
/// focus on index zero.
pub fn scenario(window: usize, startable: usize) -> Scenario {
    let posts: Vec<PostId> = (0..window).map(|i| PostId::new(format!("p{i}"))).collect();
    let mut catalog = Catalog::new();
    let mut present = PresentRanges::new();
    for (index, post) in posts.iter().enumerate() {
        catalog.upsert(post.clone(), progressive_meta(Some(1_000), Some(1_000)));
        if index < startable {
            present.set(post.clone(), vec![ByteRange::new(0, 1_000)]);
        }
    }
    let mut focus = FocusState::new();
    focus.update_focus(FocusUpdate {
        window: posts.clone(),
        current_index: 0,
        watch_ms: 0,
    });
    Scenario {
        catalog,
        focus,
        present,
        posts,
    }
}
