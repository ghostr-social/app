use crate::catalog::Catalog;
use crate::focus::{FocusState, FocusUpdate};
use crate::inventory_controller::PresentRanges;
use crate::tests::support::progressive_meta;
use crate::{ByteRange, PostId};

pub(super) struct Scenario {
    pub(super) catalog: Catalog,
    pub(super) focus: FocusState,
    pub(super) present: PresentRanges,
    pub(super) posts: Vec<PostId>,
}

/// A window of `window` tiny posts (1000 bytes, 1 s → the head is the
/// whole file), the first `startable` of them fully on disk, and the
/// focus on index zero.
pub(super) fn scenario(window: usize, startable: usize) -> Scenario {
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
