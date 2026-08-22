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

    pub(crate) const fn effective(self) -> usize {
        match self.expansion_allowed {
            true => self.count,
            false => 0,
        }
    }
}
