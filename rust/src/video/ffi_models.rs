use crate::video::native_models::{
    NativeUserData, NativeVideo, NativeVideoDelivery, NativeVideoDownload,
};

#[derive(Debug, Clone)]
pub struct FfiNostrEventIdentity {
    pub event_id: String,
    pub author_public_key_hex: String,
    pub kind: u64,
    pub identifier: Option<String>,
    pub created_at: u64,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct FfiUserData {
    pub npub: Option<String>,
    pub name: Option<String>,
    pub profile_picture: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfiVideoDelivery {
    Progressive,
    Hls,
}

#[derive(Debug, Clone)]
pub struct FfiNostrVideo {
    pub id: String,
    pub user: FfiUserData,
    pub title: String,
    pub song_name: String,
    pub likes: String,
    pub comments: String,
    pub url: String,
    pub delivery: FfiVideoDelivery,
}

#[derive(Debug, Clone)]
pub struct FfiVideoDownload {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub local_path: Option<String>,
    pub event: FfiNostrEventIdentity,
    pub nostr: FfiNostrVideo,
}

pub fn ffi_video_download(video: &NativeVideoDownload) -> FfiVideoDownload {
    FfiVideoDownload {
        id: video.id.clone(),
        url: video.url.clone(),
        title: Some(video.nostr.title.clone()),
        local_path: available_local_path(video),
        event: video.event.clone(),
        nostr: ffi_nostr_video(&video.nostr),
    }
}

fn available_local_path(video: &NativeVideoDownload) -> Option<String> {
    if video.downloading {
        return None;
    }
    video
        .local_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
}

fn ffi_nostr_video(video: &NativeVideo) -> FfiNostrVideo {
    FfiNostrVideo {
        id: video.id.clone(),
        user: ffi_user_data(&video.user),
        title: video.title.clone(),
        song_name: video.song_name.clone(),
        likes: video.likes.clone(),
        comments: video.comments.clone(),
        url: video.url.clone(),
        delivery: ffi_video_delivery(video.delivery),
    }
}

fn ffi_video_delivery(delivery: NativeVideoDelivery) -> FfiVideoDelivery {
    match delivery {
        NativeVideoDelivery::Progressive => FfiVideoDelivery::Progressive,
        NativeVideoDelivery::Hls => FfiVideoDelivery::Hls,
    }
}

fn ffi_user_data(user: &NativeUserData) -> FfiUserData {
    FfiUserData {
        npub: user.npub.clone(),
        name: user.name.clone(),
        profile_picture: user.profile_picture.clone(),
    }
}
