use super::super::{AssetBinding, AssetPlan, AssetRegistry, BindingPlan};
use super::origin_generation;
use ghostr_delivery::segmented::CachedHlsObject;
use reqwest::Url;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[tokio::test]
async fn provider_binding_is_sticky_and_cache_change_retires() {
    let url = Url::parse("https://media.example/segment.m4s").expect("valid test fixture");
    let first = object(&url, b"aaaa").generation();
    let changed = object(&url, b"bbbb").generation();
    let mut registry = AssetRegistry::new();
    let fence = registry.fence(&url, 1).expect("valid test fixture");
    let deadline = Instant::now() + Duration::from_secs(1);

    assert!(matches!(
        fence.plan(Some(first), deadline).await,
        Ok(AssetPlan::Cache(_))
    ));
    assert!(matches!(
        fence.plan(Some(first), deadline).await,
        Ok(AssetPlan::Cache(_))
    ));
    assert!(fence.plan(Some(changed), deadline).await.is_err());
    assert!(fence.plan(Some(first), deadline).await.is_err());

    let origin = origin_generation(&url, "\"v1\"", 16);
    let mut binding = AssetBinding::Origin(origin.clone());
    assert!(matches!(
        binding.next(Some(changed)),
        Ok(BindingPlan::Origin(found)) if found == origin
    ));
}

fn object(url: &Url, body: &[u8]) -> CachedHlsObject {
    CachedHlsObject::new(Arc::from(body), url.clone(), None)
}
