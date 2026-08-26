#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HlsDemand {
    count: usize,
    expansion_allowed: bool,
}

impl HlsDemand {
    pub(crate) const fn new(count: usize, expansion_allowed: bool) -> Self {
        Self {
            count,
            expansion_allowed,
        }
    }

    pub(super) const fn effective(self) -> usize {
        if self.expansion_allowed {
            self.count
        } else {
            0
        }
    }
}
