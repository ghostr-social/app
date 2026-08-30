#[derive(Debug, Clone)]
pub struct FfiHlsPreparedAssetAuthority {
    pub delivery_id: String,
    pub representation_id: String,
    pub asset_revision: u64,
}

#[derive(Debug, Clone)]
pub struct FfiHlsPlaybackSession {
    pub session_id: String,
    pub playback_url: String,
    pub delivery_id: Option<String>,
    pub representation_id: Option<String>,
    pub asset_revision: Option<u64>,
}

pub(crate) fn ffi_hls_playback_session(
    session: ghostr_gateway::hls::playback::NativeHlsPlaybackSession,
) -> FfiHlsPlaybackSession {
    let authority = session.authority.as_ref();
    FfiHlsPlaybackSession {
        session_id: session.id.as_str().to_owned(),
        playback_url: session.playback_url,
        delivery_id: authority.map(|value| value.post().as_str().to_owned()),
        representation_id: authority
            .map(|value| value.representation_id().fingerprint().to_owned()),
        asset_revision: authority.map(|value| value.asset_revision().value()),
    }
}
