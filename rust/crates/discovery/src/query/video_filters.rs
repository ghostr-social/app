//! Pure relay-filter builders for the Rust video-discovery contract:
//! canonical kinds, limits, search terms, and tag filters.

use nostr_sdk::{Alphabet, Filter, Kind, PublicKey, SingleLetterTag, Timestamp};

use crate::cache::ViewerScope;
use crate::query::hashtags::hashtag_filter_values;

/// Every NIP-71 video kind: normal + short, current + deprecated addressable.
pub(crate) const VIDEO_EVENT_KINDS: [u16; 4] = [21, 22, 34235, 34236];

/// Kind-1 notes: most Nostr videos travel as plain notes with a link.
pub(crate) const VIDEO_NOTE_KIND: u16 = 1;

/// NIP-94 file-metadata events, filtered server-side to video mimes.
pub(crate) const FILE_EVENT_KIND: u16 = 1063;

pub(crate) const REPOST_KIND: u16 = 6;
pub(crate) const GENERIC_REPOST_KIND: u16 = 16;
pub(crate) const DELETION_KIND: u16 = 5;

/// Mime values worth asking NIP-94 file events for, via the `#m` filter.
pub(crate) const VIDEO_FILE_MIME_TYPES: [&str; 6] = [
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/mpeg",
    "application/x-mpegurl",
    "application/vnd.apple.mpegurl",
];

/// NIP-50 term hunting notes that literally mention a video file, issued
/// only when the viewer gave no term of their own.
pub(crate) const VIDEO_NOTE_HUNT_TERM: &str = "mp4";

/// Video-kind query limit for a plain (non-widened) feed page.
pub(crate) const FEED_VIDEO_LIMIT: usize = 80;

/// Limit for widened video queries and every note/file query.
pub(crate) const WIDE_QUERY_LIMIT: usize = 200;

/// Whether the scheduler serves one requested page or keeps the relay
/// source alive by alternating history bursts with head refreshes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiscoveryFlow {
    #[default]
    Page,
    Continuous,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RepostAdmission {
    #[default]
    Excluded,
    Included,
}

/// One video-discovery request accepted by the Rust engine.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveryRequest {
    pub authors: Vec<PublicKey>,
    /// Authors the request is *routed* by without being filtered by:
    /// the main feed rides its follows' write relays and still shows
    /// whatever those relays carry from anyone.
    /// Never reaches the wire filter.
    pub routing_authors: Vec<PublicKey>,
    /// Viewer search term as typed; a blank term still widens the request
    /// but carries no NIP-50 term.
    pub search_query: Option<String>,
    pub hashtags: Vec<String>,
    /// Inclusive publication cutoff (`until`) for older pages.
    pub older_than: Option<Timestamp>,
    /// Whose session this request belongs to, for the event pool
    /// (`crate::cache`). Like `routing_authors` it
    /// never reaches the wire filter; only the main feed knows a viewer,
    /// so every other feed leaves the scope alone.
    pub viewer: ViewerScope,
    pub flow: DiscoveryFlow,
    pub reposts: RepostAdmission,
}

impl DiscoveryRequest {
    /// Widened requests carry a viewer term or hashtags.
    fn is_wide(&self) -> bool {
        self.search_query.is_some() || !self.hashtags.is_empty()
    }

    pub(crate) fn is_continuous(&self) -> bool {
        self.flow == DiscoveryFlow::Continuous || self.is_wide()
    }

    /// The authors this request routes to: the ones it filters by, or —
    /// when it filters by nobody — the routing-only set.
    pub(crate) fn routed_authors(&self) -> &[PublicKey] {
        if self.authors.is_empty() {
            &self.routing_authors
        } else {
            &self.authors
        }
    }

    /// Trimmed NIP-50 term; blank input carries no term.
    pub(crate) fn normalized_search(&self) -> Option<&str> {
        let term = self.search_query.as_deref()?.trim();
        if term.is_empty() {
            None
        } else {
            Some(term)
        }
    }
}

/// Builds the filters in canonical order: dedicated video kinds first,
/// then the additive note, note-hunt, and file queries.
pub(crate) fn discovery_filters(request: &DiscoveryRequest) -> Vec<Filter> {
    let mut filters = vec![video_kinds_filter(request), note_filter(request)];
    if request.search_query.is_none() {
        filters.push(note_hunt_filter(request));
    }
    filters.push(file_event_filter(request));
    if request.reposts == RepostAdmission::Included {
        filters.extend(repost_filters(request));
    }
    filters
}

fn repost_filters(request: &DiscoveryRequest) -> [Filter; 3] {
    [
        scoped(request, &[REPOST_KIND], WIDE_QUERY_LIMIT),
        scoped(request, &[GENERIC_REPOST_KIND], WIDE_QUERY_LIMIT),
        scoped(request, &[DELETION_KIND], WIDE_QUERY_LIMIT),
    ]
}

/// Dedicated NIP-71 video kinds; narrow limit unless the request widened.
fn video_kinds_filter(request: &DiscoveryRequest) -> Filter {
    let limit = if request.is_wide() {
        WIDE_QUERY_LIMIT
    } else {
        FEED_VIDEO_LIMIT
    };
    let filter = scoped(request, &VIDEO_EVENT_KINDS, limit);
    with_search(filter, request.normalized_search())
}

/// Kind-1 note window paired with every request.
fn note_filter(request: &DiscoveryRequest) -> Filter {
    let filter = scoped(request, &[VIDEO_NOTE_KIND], WIDE_QUERY_LIMIT);
    with_search(filter, request.normalized_search())
}

/// NIP-50 hunt for notes mentioning a video file; only built when the
/// viewer gave no term of their own.
fn note_hunt_filter(request: &DiscoveryRequest) -> Filter {
    scoped(request, &[VIDEO_NOTE_KIND], WIDE_QUERY_LIMIT).search(VIDEO_NOTE_HUNT_TERM)
}

/// NIP-94 file query filtered server-side to video mimes.
fn file_event_filter(request: &DiscoveryRequest) -> Filter {
    let filter = scoped(request, &[FILE_EVENT_KIND], WIDE_QUERY_LIMIT).custom_tag(
        SingleLetterTag::lowercase(Alphabet::M),
        VIDEO_FILE_MIME_TYPES,
    );
    with_search(filter, request.normalized_search())
}

fn scoped(request: &DiscoveryRequest, kinds: &[u16], limit: usize) -> Filter {
    let mut filter = Filter::new()
        .kinds(kinds.iter().copied().map(Kind::from))
        .limit(limit);
    if !request.authors.is_empty() {
        filter = filter.authors(request.authors.iter().copied());
    }
    if let Some(until) = request.older_than {
        filter = filter.until(until);
    }
    with_hashtags(filter, request)
}

fn with_hashtags(filter: Filter, request: &DiscoveryRequest) -> Filter {
    let values = hashtag_filter_values(&request.hashtags);
    if values.is_empty() {
        return filter;
    }
    filter.custom_tag(SingleLetterTag::lowercase(Alphabet::T), values)
}

fn with_search(filter: Filter, term: Option<&str>) -> Filter {
    match term {
        Some(term) => filter.search(term),
        None => filter,
    }
}
