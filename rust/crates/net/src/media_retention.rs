//! Conservative retention projection; freshness and source validation are
//! independent requirements. Ambiguous policy never authorizes derived reuse.
use reqwest::header::{HeaderMap, CACHE_CONTROL, SET_COOKIE, VARY};
use reqwest::Url;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaRetention {
    Public,
    Partitioned,
    Transient,
}

impl MediaRetention {
    pub fn from_headers(headers: &HeaderMap, url: &Url) -> Self {
        if contextual(headers, url) {
            return Self::Transient;
        }
        let mut public = false;
        for value in headers.get_all(CACHE_CONTROL) {
            let Ok(value) = value.to_str() else {
                return Self::Transient;
            };
            if value.contains('"') || value.len() > 8_192 {
                return Self::Transient;
            }
            for directive in value.split(',') {
                match directive
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "no-store" | "no-cache" | "private" => return Self::Transient,
                    "public" => public = true,
                    _ => {}
                }
            }
        }
        if public {
            Self::Public
        } else {
            Self::Partitioned
        }
    }
}

fn contextual(headers: &HeaderMap, url: &Url) -> bool {
    headers.contains_key(VARY)
        || headers.contains_key(SET_COOKIE)
        || url.query().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
}
