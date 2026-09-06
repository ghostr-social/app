use core::fmt::{Display, Formatter};

#[derive(Debug)]
pub(crate) struct BodyLeaseDenied;

impl Display for BodyLeaseDenied {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("continuous body has no current network lease")
    }
}

impl core::error::Error for BodyLeaseDenied {}
