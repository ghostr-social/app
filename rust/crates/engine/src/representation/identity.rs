use crate::{DeliveryKind, VideoMeta};
use sha2::{Digest, Sha256};

pub(super) fn fingerprint(meta: &VideoMeta) -> String {
    let mut digest = Sha256::new();
    digest.update([delivery_tag(meta.delivery)]);
    match &meta.sha256 {
        Some(advertised) => field(&mut digest, advertised.as_bytes()),
        None => hash_unverified(&mut digest, meta),
    }
    format!("{:x}", digest.finalize())
}

fn hash_unverified(digest: &mut Sha256, meta: &VideoMeta) {
    let mut urls = meta.urls.clone();
    urls.sort();
    urls.dedup();
    for url in urls {
        field(digest, url.as_bytes());
    }
    optional_number(digest, meta.size_bytes);
    optional_number(digest, meta.duration_ms);
}

fn field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn optional_number(digest: &mut Sha256, value: Option<u64>) {
    digest.update([u8::from(value.is_some())]);
    digest.update(value.unwrap_or_default().to_be_bytes());
}

fn delivery_tag(delivery: DeliveryKind) -> u8 {
    match delivery {
        DeliveryKind::Progressive => 0,
        DeliveryKind::Hls => 1,
    }
}
