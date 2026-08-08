//! The shared nouns of the retrieval pipeline: which screen a fetch
//! serves, how urgent it is, how it failed, and what it reported back.
//! Transport, routing, and scheduling all speak this vocabulary, so it
//! is kept dependency-free and each layer can depend on it without
//! depending on the others.

use crate::session_generation::SessionGeneration;
use nostr_sdk::Event;
use tokio::sync::mpsc;

/// Screen-level scope a retrieval serves, e.g. `feed`, `search:ghost`,
/// `tag:dance`, or `discover`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FeedContext {
    value: String,
    session: SessionGeneration,
}

impl FeedContext {
    pub fn for_session(value: impl Into<String>, session: SessionGeneration) -> Self {
        Self {
            value: value.into(),
            session,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }

    pub fn session(&self) -> SessionGeneration {
        self.session
    }
}

/// Priority classes for content retrieval, most urgent first.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RetrievalPriority {
    Interactive,
    Enrichment,
    Background,
}

/// Describes one unit of network retrieval for scheduling decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrievalRequest {
    pub(crate) context: FeedContext,
    pub(crate) priority: RetrievalPriority,
}

/// Why a whole retrieval could not settle. Any planned content-query failure
/// leaves the page retryable so pagination never commits a partial boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanFailure {
    pub message: String,
}

impl PlanFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Events yielded before a retrieval reaches EOSE/timeout and profile
/// enrichment. The scheduler turns them into provisional feed updates.
pub type EventProgress = mpsc::Sender<Event>;

/// One executed retrieval's role in feed assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalPurpose {
    Head,
    Older,
}

#[derive(Clone, Debug)]
pub enum RetrievalOutcome {
    Started {
        context: FeedContext,
    },
    Progress {
        context: FeedContext,
        event: Box<Event>,
    },
    Completed {
        context: FeedContext,
        result: Result<Vec<Event>, PlanFailure>,
        purpose: RetrievalPurpose,
    },
}
