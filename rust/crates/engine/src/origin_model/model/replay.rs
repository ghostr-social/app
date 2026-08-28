use super::super::keys::{OriginContextKey, UrlContextKey};
use super::super::{MediaClass, OriginContext, OriginQuery, RequestMethod};
use super::OriginModel;
use crate::host_stats::host_of;
use std::collections::BTreeMap;

impl OriginModel {
    pub(crate) fn replay_project(
        &self,
        aliases: &[(String, String)],
        opaque: &impl Fn(&str) -> String,
    ) -> Self {
        let aliases = Aliases::new(aliases);
        Self {
            global: self.global.clone(),
            origins: project_origins(&self.origins, &aliases, opaque),
            urls: project_urls(&self.urls, &aliases, opaque),
            open_body_global: self.open_body_global.clone(),
            open_body_origins: project_origins(&self.open_body_origins, &aliases, opaque),
            open_body_urls: project_urls(&self.open_body_urls, &aliases, opaque),
            circuits: self
                .circuits
                .replay_project(|origin| aliases.origins(origin, opaque)),
            priors: self.priors.clone(),
            exploration: self
                .exploration
                .replay_project(|origin| aliases.origins(origin, opaque)),
        }
    }

    pub(crate) fn replay_bounded(&self) -> bool {
        self.global.len() <= super::GLOBAL_CAP
            && self.origins.len() <= super::ORIGIN_CAP
            && self.urls.len() <= super::URL_CAP
            && self.open_body_global.len() <= super::GLOBAL_CAP
            && self.open_body_origins.len() <= super::ORIGIN_CAP
            && self.open_body_urls.len() <= super::URL_CAP
            && self.priors.len() <= super::PRIOR_CAP
            && self.circuits.replay_bounded()
            && self.exploration.replay_bounded()
    }
}

struct Aliases {
    origins: BTreeMap<String, Vec<String>>,
    urls: BTreeMap<String, Vec<String>>,
}

impl Aliases {
    fn new(values: &[(String, String)]) -> Self {
        let mut aliases = Self {
            origins: BTreeMap::new(),
            urls: BTreeMap::new(),
        };
        for (raw, projected) in values {
            insert_alias(&mut aliases.origins, host(raw), host(projected));
            insert_alias(&mut aliases.urls, url_id(raw), url_id(projected));
        }
        aliases
    }

    fn origins(&self, value: &str, opaque: &impl Fn(&str) -> String) -> Vec<String> {
        self.origins
            .get(value)
            .cloned()
            .unwrap_or_else(|| vec![format!("{}.invalid", opaque(value))])
    }

    fn urls(&self, value: &str, opaque: &impl Fn(&str) -> String) -> Vec<String> {
        self.urls
            .get(value)
            .cloned()
            .unwrap_or_else(|| vec![opaque(value)])
    }
}

fn project_origins(
    values: &BTreeMap<OriginContextKey, super::super::AdaptiveRecord>,
    aliases: &Aliases,
    opaque: &impl Fn(&str) -> String,
) -> BTreeMap<OriginContextKey, super::super::AdaptiveRecord> {
    values
        .iter()
        .flat_map(|(key, record)| {
            aliases
                .origins(&key.origin, opaque)
                .into_iter()
                .map(|origin| {
                    (
                        OriginContextKey {
                            origin,
                            context: key.context,
                        },
                        record.clone(),
                    )
                })
        })
        .collect()
}

fn project_urls(
    values: &BTreeMap<UrlContextKey, super::super::AdaptiveRecord>,
    aliases: &Aliases,
    opaque: &impl Fn(&str) -> String,
) -> BTreeMap<UrlContextKey, super::super::AdaptiveRecord> {
    values
        .iter()
        .flat_map(|(key, record)| {
            aliases.urls(&key.url_id, opaque).into_iter().map(|url_id| {
                (
                    UrlContextKey {
                        url_id,
                        context: key.context,
                    },
                    record.clone(),
                )
            })
        })
        .collect()
}

fn insert_alias(map: &mut BTreeMap<String, Vec<String>>, raw: String, projected: String) {
    let values = map.entry(raw).or_default();
    if !values.contains(&projected) {
        values.push(projected);
    }
}

fn host(value: &str) -> String {
    host_of(value).unwrap_or_else(|| "unavailable".into())
}

fn url_id(value: &str) -> String {
    let context = OriginContext::new(RequestMethod::Head, 0, MediaClass::Unknown);
    OriginQuery::new(value, context).url_id().to_owned()
}
