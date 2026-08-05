#[derive(Debug, Clone)]
pub struct FfiHlsPlaybackSession {
    pub session_id: String,
    pub playback_url: String,
}

pub fn ffi_hls_playback_session(
    session: crate::video::hls_playback_gateway::NativeHlsPlaybackSession,
) -> FfiHlsPlaybackSession {
    FfiHlsPlaybackSession {
        session_id: session.id.as_str().to_owned(),
        playback_url: session.playback_url,
    }
}
