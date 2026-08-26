//! Author-ordered NIP-B7 Blossom mirrors from replaceable kind-10063 lists.

use super::parsing::ParsedVideoPost;
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::VideoMeta;
use ghostr_media_model::blossom::terminal_sha256;
use nostr_sdk::{Event, EventId, Timestamp};
use std::collections::HashMap;
use url::Url;

const SERVER_LIST_KIND: u16 = 10063;
const MAX_SERVER_URL_BYTES: usize = 2_048;

#[derive(Debug, Default)]
pub(crate) struct BlossomServerStore {
    lists: HashMap<String, ServerList>,
}

#[derive(Debug)]
struct ServerList {
    created_at: Timestamp,
    event_id: EventId,
    servers: Vec<String>,
}

impl BlossomServerStore {
    pub(super) fn ingest(&mut self, events: &[Event]) {
        for event in events
            .iter()
            .filter(|event| event.kind.as_u16() == SERVER_LIST_KIND)
        {
            let Some(candidate) = server_list(event) else {
                continue;
            };
            let author = event.pubkey.to_hex();
            if self
                .lists
                .get(&author)
                .is_none_or(|current| newer(&candidate, current))
            {
                self.lists.insert(author, candidate);
            }
        }
    }

    pub(super) fn enrich(&self, post: &mut ParsedVideoPost) {
        let Some(list) = self.lists.get(&post.author_pubkey) else {
            return;
        };
        add_mirrors(&mut post.meta, &list.servers);
        post.renditions = post
            .renditions
            .iter()
            .map(|rendition| enrich_rendition(rendition, &list.servers))
            .collect();
        enrich_evidence(post, &list.servers);
    }

    pub(super) fn clear(&mut self) {
        self.lists.clear();
    }
}

pub(crate) fn supports_blossom(post: &ParsedVideoPost) -> bool {
    exact_identity(&post.meta).is_some()
        || post
            .renditions
            .iter()
            .any(|rendition| exact_identity(rendition.meta()).is_some())
}

fn enrich_rendition(rendition: &VideoRendition, servers: &[String]) -> VideoRendition {
    let bitrate = rendition.bitrate_bits_per_second();
    let mut meta = rendition.meta().clone();
    add_mirrors(&mut meta, servers);
    VideoRendition::try_new(meta, bitrate).expect("an enriched rendition stays progressive")
}

fn add_mirrors(meta: &mut VideoMeta, servers: &[String]) {
    let Some(digest) = exact_identity(meta) else {
        return;
    };
    for server in servers {
        let url = format!("{server}/{digest}");
        if !meta.urls.iter().any(|current| current == &url) {
            meta.urls.push(url);
        }
    }
}

fn enrich_evidence(post: &mut ParsedVideoPost, servers: &[String]) {
    for evidence in &mut post.metadata_evidence {
        let digest = evidence
            .sha256
            .clone()
            .or_else(|| evidence.urls.iter().find_map(|url| terminal_sha256(url)));
        let Some(digest) = digest.filter(|digest| urls_match_identity(&evidence.urls, digest))
        else {
            continue;
        };
        evidence.sha256 = Some(digest.clone());
        for server in servers {
            let url = format!("{server}/{digest}");
            if !evidence.urls.iter().any(|current| current == &url) {
                evidence.urls.push(url);
            }
        }
    }
}

fn exact_identity(meta: &VideoMeta) -> Option<String> {
    let digest = meta.sha256.as_ref()?;
    urls_match_identity(&meta.urls, digest).then(|| digest.clone())
}

fn urls_match_identity(urls: &[String], digest: &str) -> bool {
    urls.iter()
        .filter_map(|url| terminal_sha256(url))
        .all(|found| found.eq_ignore_ascii_case(digest))
}

fn server_list(event: &Event) -> Option<ServerList> {
    let mut servers = Vec::new();
    for tag in event.tags.iter() {
        let values = tag.as_slice();
        if values.first().map(String::as_str) != Some("server") {
            continue;
        }
        let Some(server) = values.get(1).and_then(|raw| normalized_server(raw)) else {
            continue;
        };
        if !servers.contains(&server) {
            servers.push(server);
        }
    }
    (!servers.is_empty()).then_some(ServerList {
        created_at: event.created_at,
        event_id: event.id,
        servers,
    })
}

fn normalized_server(raw: &str) -> Option<String> {
    if raw.len() > MAX_SERVER_URL_BYTES {
        return None;
    }
    let parsed = Url::parse(raw).ok()?;
    let safe = matches!(parsed.scheme(), "http" | "https")
        && parsed.host().is_some()
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.query().is_none()
        && parsed.fragment().is_none();
    safe.then(|| parsed.as_str().trim_end_matches('/').to_owned())
}

fn newer(candidate: &ServerList, current: &ServerList) -> bool {
    candidate.created_at > current.created_at
        || (candidate.created_at == current.created_at && candidate.event_id < current.event_id)
}
