//! Where the viewer is: the ordered window of posts around the current
//! item, plus how long the current item has been watched.

use crate::engine::PostId;

/// A full replacement of the focus window, as sent over the FFI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusUpdate {
    pub window: Vec<PostId>,
    pub current_index: usize,
    pub watch_ms: u64,
}

/// The engine's view of viewer attention. The window includes the
/// current item; `current_index` points at it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FocusState {
    window: Vec<PostId>,
    current_index: usize,
    watch_ms: u64,
}

impl FocusState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the window wholesale. An out-of-bounds index clamps to
    /// the last item; an empty window resets to index zero.
    pub fn update_focus(&mut self, update: FocusUpdate) {
        self.current_index = clamp_index(update.current_index, update.window.len());
        self.window = update.window;
        self.watch_ms = update.watch_ms;
    }

    pub fn window(&self) -> &[PostId] {
        &self.window
    }

    pub fn current(&self) -> Option<&PostId> {
        self.window.get(self.current_index)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn watch_ms(&self) -> u64 {
        self.watch_ms
    }

    /// Signed scroll distance: 0 = current, positive = ahead, negative =
    /// behind. `None` when the post is outside the window.
    pub fn distance_of(&self, post: &PostId) -> Option<i64> {
        let position = self.window.iter().position(|item| item == post)?;
        Some(position as i64 - self.current_index as i64)
    }

    /// Whether the viewer has watched past the commitment threshold
    /// (plan §3: commitment means the engine should finish this video).
    pub fn is_committed(&self, commitment_ms: u64) -> bool {
        self.watch_ms >= commitment_ms
    }
}

fn clamp_index(index: usize, len: usize) -> usize {
    match len {
        0 => 0,
        _ => index.min(len - 1),
    }
}
