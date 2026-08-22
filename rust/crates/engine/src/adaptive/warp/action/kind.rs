use crate::adaptive::HlsBootstrapStage;
use crate::{ActionId, ByteRange};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TransformKind {
    Remux,
    Segment,
    Transcode,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ActionKind {
    Head,
    Prefix(ByteRange),
    Tail(ByteRange),
    FetchRange(ByteRange),
    FetchWhole {
        maximum_bytes: u64,
    },
    HlsBootstrap {
        stage: HlsBootstrapStage,
        maximum_bytes: u64,
    },
    Promote {
        active: ActionId,
        maximum_bytes: u64,
    },
    Transform(TransformKind),
    CacheUpgrade(ByteRange),
    Hedge {
        primary: ActionId,
        alternate: String,
    },
    Cancel(ActionId),
}
