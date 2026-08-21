use crate::host_stats::host_of;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RequestOccupancy {
    total: usize,
    authorities: BTreeMap<String, usize>,
}

impl RequestOccupancy {
    pub fn from_sources<'a>(sources: impl IntoIterator<Item = &'a str>) -> Self {
        let mut occupancy = Self::default();
        for source in sources {
            occupancy.occupy(source);
        }
        occupancy
    }

    pub const fn total(&self) -> usize {
        self.total
    }

    pub(super) fn authority_count(&self, source: &str) -> usize {
        self.authorities
            .get(&request_authority(source))
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn authorities(&self) -> &BTreeMap<String, usize> {
        &self.authorities
    }

    fn occupy(&mut self, source: &str) {
        self.total = self.total.saturating_add(1);
        let used = self
            .authorities
            .entry(request_authority(source))
            .or_default();
        *used = used.saturating_add(1);
    }
}

pub(super) fn request_authority(source: &str) -> String {
    host_of(source).unwrap_or_else(|| source.to_owned())
}
