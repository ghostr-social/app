//! Where the viewer is: the ordered window of posts around the current
//! item, plus how long the current item has been watched.

use crate::PostId;

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
    }

    pub fn window(&self) -> &[PostId] {
        &self.window
    }

    pub fn current(&self) -> Option<&PostId> {
        self.window.get(self.current_index)
    }
}

fn clamp_index(index: usize, len: usize) -> usize {
    match len {
        0 => 0,
        _ => index.min(len - 1),
    }
}
