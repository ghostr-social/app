use super::*;

impl CooldownTimers {
    pub(crate) fn len(&self) -> usize {
        self.active.len()
    }
}
