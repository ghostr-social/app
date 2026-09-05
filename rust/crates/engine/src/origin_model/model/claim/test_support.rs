use super::{Admission, ClaimedAdmission};

impl ClaimedAdmission {
    pub const fn admission(&self) -> Admission {
        self.admission
    }
}
