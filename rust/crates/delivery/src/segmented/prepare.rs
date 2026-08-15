use super::fetch::{asset, manifest, FetchedObject};
use anyhow::{bail, Result};
use ghostr_hls_manifest::hls_manifest::{inspect_hls_bootstrap, HlsBootstrap};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::sync::Arc;
use url::Url;

pub(crate) struct PreparedHls {
    pub(super) objects: Vec<PreparedObject>,
}

pub(super) struct PreparedObject {
    pub request_url: String,
    pub final_url: Url,
    pub body: Arc<[u8]>,
    pub content_type: Option<String>,
}

impl PreparedHls {
    pub(super) fn bytes_present(&self) -> u64 {
        self.objects
            .iter()
            .map(|object| object.body.len() as u64)
            .sum()
    }
}

pub(crate) async fn prepare_hls(
    client: &dyn MediaHttpRequests,
    sources: &[String],
) -> Result<PreparedHls> {
    let mut last = None;
    for source in sources {
        match prepare_source(client, source).await {
            Ok(prepared) => return Ok(prepared),
            Err(error) => last = Some(error.context(format!("HLS source {source}"))),
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("HLS item has no source")))
}

async fn prepare_source(client: &dyn MediaHttpRequests, source: &str) -> Result<PreparedHls> {
    let root = manifest(client, source).await?;
    let inspected = inspect_hls_bootstrap(&root.body, &root.final_url)?;
    let mut objects = vec![root.into()];
    let media = match inspected {
        HlsBootstrap::Media { init, segment } => (init, segment),
        HlsBootstrap::Master { variant } => {
            let child = manifest(client, variant.as_str()).await?;
            let inspected = inspect_hls_bootstrap(&child.body, &child.final_url)?;
            objects.push(child.into());
            match inspected {
                HlsBootstrap::Media { init, segment } => (init, segment),
                HlsBootstrap::Master { .. } => bail!("nested HLS master exceeds depth limit"),
            }
        }
    };
    if let Some(init) = media.0 {
        objects.push(asset(client, &init).await?.into());
    }
    objects.push(asset(client, &media.1).await?.into());
    Ok(PreparedHls { objects })
}

impl From<FetchedObject> for PreparedObject {
    fn from(object: FetchedObject) -> Self {
        Self {
            request_url: object.request_url,
            final_url: object.final_url,
            body: object.body,
            content_type: object.content_type,
        }
    }
}
