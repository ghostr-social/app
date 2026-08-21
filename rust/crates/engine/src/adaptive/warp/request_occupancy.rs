use crate::RequestAuthority;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RequestOccupancy {
    total: usize,
    authorities: BTreeMap<RequestAuthority, usize>,
    invalid: usize,
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
        let Some(authority) = RequestAuthority::from_url(source) else {
            return self.invalid;
        };
        self.authorities
            .get(&authority)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn authorities(&self) -> &BTreeMap<RequestAuthority, usize> {
        &self.authorities
    }

    fn occupy(&mut self, source: &str) {
        self.total = self.total.saturating_add(1);
        match RequestAuthority::from_url(source) {
            Some(authority) => {
                let used = self.authorities.entry(authority).or_default();
                *used = used.saturating_add(1);
            }
            None => self.invalid = self.invalid.saturating_add(1),
        }
    }

    pub(crate) fn replay_project(&self, source: &impl Fn(&str) -> String) -> Self {
        let authorities = self
            .authorities
            .iter()
            .filter_map(|(authority, count)| {
                RequestAuthority::from_url(&source(authority.as_str()))
                    .map(|projected| (projected, *count))
            })
            .collect();
        Self {
            total: self.total,
            authorities,
            invalid: self.invalid,
        }
    }

    pub(crate) fn replay_bounded(&self, limit: usize) -> bool {
        self.authorities.len() <= limit
    }
}
