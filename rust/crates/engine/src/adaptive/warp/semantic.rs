use crate::PostId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticScore {
    Known(u64),
    Unavailable { rank: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticCandidate {
    pub post: PostId,
    pub score: SemanticScore,
    pub ready: bool,
}

impl SemanticCandidate {
    pub const fn new(post: PostId, score: SemanticScore, ready: bool) -> Self {
        Self { post, score, ready }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportCensorReason {
    OriginFailure,
    PolicyRejection,
    UnavoidableReadinessFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticAdmission {
    pub admissible: bool,
    pub rescue: bool,
    pub rank_displacement: usize,
    pub censor: Option<TransportCensorReason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticGuardrail {
    top_k: usize,
    epsilon_micros: u64,
}

impl SemanticGuardrail {
    pub const fn new(top_k: usize, epsilon_micros: u64) -> Self {
        Self {
            top_k,
            epsilon_micros,
        }
    }

    pub fn admit(
        self,
        candidate: &SemanticCandidate,
        window: &[SemanticCandidate],
    ) -> SemanticAdmission {
        let rank = window.iter().position(|item| item.post == candidate.post);
        let Some(rank) = rank else { return rejected() };
        if self.normal(candidate, window, rank) {
            return accepted(rank);
        }
        if window
            .iter()
            .any(|item| item.ready && self.is_normal(item, window))
        {
            return rejected();
        }
        rescued(rank)
    }

    fn normal(
        self,
        candidate: &SemanticCandidate,
        window: &[SemanticCandidate],
        rank: usize,
    ) -> bool {
        rank < self.top_k.max(1) && self.is_normal(candidate, window)
    }

    fn is_normal(self, candidate: &SemanticCandidate, window: &[SemanticCandidate]) -> bool {
        match candidate.score {
            SemanticScore::Known(score) => best_score(window)
                .is_some_and(|best| score.saturating_add(self.epsilon_micros) >= best),
            SemanticScore::Unavailable { rank } => rank < self.top_k.max(1),
        }
    }
}

fn best_score(window: &[SemanticCandidate]) -> Option<u64> {
    window
        .iter()
        .filter_map(|item| match item.score {
            SemanticScore::Known(score) => Some(score),
            SemanticScore::Unavailable { .. } => None,
        })
        .max()
}

fn accepted(rank: usize) -> SemanticAdmission {
    SemanticAdmission {
        admissible: true,
        rescue: false,
        rank_displacement: rank,
        censor: None,
    }
}

fn rescued(rank: usize) -> SemanticAdmission {
    SemanticAdmission {
        admissible: true,
        rescue: true,
        rank_displacement: rank,
        censor: Some(TransportCensorReason::UnavoidableReadinessFailure),
    }
}

fn rejected() -> SemanticAdmission {
    SemanticAdmission {
        admissible: false,
        rescue: false,
        rank_displacement: 0,
        censor: None,
    }
}
