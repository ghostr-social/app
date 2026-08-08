use crate::hls::sessions::HlsResourceId;
use anyhow::{ensure, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ghostr_hls_manifest::hls_manifest::{HlsResource, HlsResourceKind};
use hmac::{Hmac, Mac};
use reqwest::Url;
use sha2::Sha256;

const TOKEN_VERSION: u8 = 1;
const MAC_BYTES: usize = 32;
const MAX_RESOURCE_URL_BYTES: usize = 8_192;

pub(crate) fn issue(secret: &[u8; 32], resource: HlsResource) -> Result<HlsResourceId> {
    let raw_url = resource.url.as_str().as_bytes();
    ensure!(
        raw_url.len() <= MAX_RESOURCE_URL_BYTES,
        "HLS resource URL is too long"
    );
    let mut payload = Vec::with_capacity(raw_url.len() + MAC_BYTES + 2);
    payload.extend([TOKEN_VERSION, kind_byte(resource.kind)]);
    payload.extend(raw_url);
    let signature = signature(secret, &payload);
    payload.extend(signature);
    Ok(HlsResourceId(URL_SAFE_NO_PAD.encode(payload)))
}

pub(crate) fn open(secret: &[u8; 32], id: HlsResourceId) -> Option<HlsResource> {
    let decoded = URL_SAFE_NO_PAD.decode(id.0).ok()?;
    let split = decoded.len().checked_sub(MAC_BYTES)?;
    let (payload, signature) = decoded.split_at(split);
    payload
        .first()
        .filter(|version| **version == TOKEN_VERSION)?;
    let kind = resource_kind(*payload.get(1)?)?;
    verify(secret, payload, signature).ok()?;
    let url = Url::parse(std::str::from_utf8(payload.get(2..)?).ok()?).ok()?;
    valid_resource_url(&url).then_some(HlsResource { url, kind })
}

fn signature(secret: &[u8; 32], payload: &[u8]) -> [u8; MAC_BYTES] {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts every key size");
    mac.update(payload);
    mac.finalize().into_bytes().into()
}

fn verify(secret: &[u8; 32], payload: &[u8], signature: &[u8]) -> Result<()> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts every key size");
    mac.update(payload);
    mac.verify_slice(signature)?;
    Ok(())
}

fn kind_byte(kind: HlsResourceKind) -> u8 {
    match kind {
        HlsResourceKind::Manifest => 0,
        HlsResourceKind::Asset => 1,
    }
}

fn resource_kind(raw: u8) -> Option<HlsResourceKind> {
    match raw {
        0 => Some(HlsResourceKind::Manifest),
        1 => Some(HlsResourceKind::Asset),
        _ => None,
    }
}

fn valid_resource_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.host().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}
