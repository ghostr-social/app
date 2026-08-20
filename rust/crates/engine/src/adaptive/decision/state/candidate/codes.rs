use crate::adaptive::{MediaLayout, PlayerPreparation};

pub(super) fn layout_code(value: MediaLayout) -> u8 {
    match value {
        MediaLayout::Unknown => 0,
        MediaLayout::Streamable => 1,
        MediaLayout::RequiresCompleteFile => 2,
    }
}

pub(super) fn layout(value: u8) -> MediaLayout {
    match value {
        0 => MediaLayout::Unknown,
        1 => MediaLayout::Streamable,
        _ => MediaLayout::RequiresCompleteFile,
    }
}

pub(super) fn preparation_code(value: PlayerPreparation) -> u8 {
    match value {
        PlayerPreparation::Unverified => 0,
        PlayerPreparation::Initializing => 1,
        PlayerPreparation::PluginReady => 2,
        PlayerPreparation::FirstFrameRendered => 3,
        PlayerPreparation::Failed => 4,
    }
}

pub(super) fn preparation(value: u8) -> PlayerPreparation {
    match value {
        0 => PlayerPreparation::Unverified,
        1 => PlayerPreparation::Initializing,
        2 => PlayerPreparation::PluginReady,
        3 => PlayerPreparation::FirstFrameRendered,
        _ => PlayerPreparation::Failed,
    }
}
