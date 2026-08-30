use crate::hls::asset_generation::{AssetFence, AssetPlan};
use crate::hls::asset_request::AssetRangeRequest;
use crate::hls::asset_response::{self, validate};
use crate::hls::cached;
use crate::hls::routes::parsed_resource;
use crate::hls::sessions::{HlsPlaybackBinding, HlsResourceId, HlsSessionId};
use crate::router::GatewayHttpState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Response, StatusCode};
use ghostr_delivery::segmented::CachedHlsObject;
use ghostr_hls_manifest::hls_manifest::HlsResourceKind;
use reqwest::Url;
use std::sync::Arc;
use tokio::time::Instant;

mod origin;
use origin::OriginRequest;

struct AssetCall {
    state: Arc<GatewayHttpState>,
    session: HlsSessionId,
    resource: HlsResourceId,
    headers: HeaderMap,
    deadline: Instant,
}

pub(crate) async fn asset(
    State(state): State<Arc<GatewayHttpState>>,
    Path((raw_session, raw_resource)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    let deadline = Instant::now() + state.hls_timeouts.total;
    let (session, resource) = parsed_resource(&raw_session, &raw_resource)?;
    AssetCall {
        state,
        session,
        resource,
        headers,
        deadline,
    }
    .serve()
    .await
}

impl AssetCall {
    async fn serve(self) -> Result<Response<Body>, StatusCode> {
        let resource = self
            .state
            .hls_sessions
            .resource_at(&self.session, self.resource.clone(), self.deadline)
            .await
            .map_err(bad_gateway)?
            .filter(|item| item.kind == HlsResourceKind::Asset)
            .ok_or(StatusCode::NOT_FOUND)?;
        let range = AssetRangeRequest::collect(&self.headers);
        if !range.is_ranged() {
            return self.full(resource.url, range).await;
        }
        if range.locally_unsatisfiable() {
            return self.local_unsatisfiable(&resource.url, range).await;
        }
        self.ranged(resource.url, range).await
    }

    async fn full(&self, url: Url, range: AssetRangeRequest) -> Result<Response<Body>, StatusCode> {
        if let Some(object) = self.playback_object(&url).await? {
            return cached::response(&object, range);
        }
        let transfer = self.open(&url, range, None).await?;
        let envelope = validate(range, transfer.response()).map_err(bad_gateway)?;
        transfer.into_proxy(envelope)
    }

    async fn local_unsatisfiable(
        &self,
        url: &Url,
        range: AssetRangeRequest,
    ) -> Result<Response<Body>, StatusCode> {
        self.playback_object(url)
            .await?
            .map_or_else(asset_response::local_unsatisfiable, |object| {
                cached::response(&object, range)
            })
    }

    async fn ranged(
        &self,
        url: Url,
        range: AssetRangeRequest,
    ) -> Result<Response<Body>, StatusCode> {
        let object = self.playback_object(&url).await?;
        let fence = self.fence(&url).await?;
        let plan = fence
            .plan(
                object.as_ref().map(CachedHlsObject::generation),
                self.deadline,
            )
            .await
            .map_err(bad_gateway)?;
        self.ensure_owner(&fence).await?;
        match plan {
            AssetPlan::Cache(generation) => Self::cached(object, generation, range),
            AssetPlan::First(admission) => {
                self.first(OriginRequest { url, range, fence }, admission)
                    .await
            }
            AssetPlan::Origin(generation) => {
                self.continue_origin(OriginRequest { url, range, fence }, generation)
                    .await
            }
        }
    }

    fn cached(
        object: Option<CachedHlsObject>,
        generation: ghostr_delivery::segmented::CachedHlsGeneration,
        range: AssetRangeRequest,
    ) -> Result<Response<Body>, StatusCode> {
        let object = object.filter(|object| object.generation() == generation);
        cached::response(&object.ok_or(StatusCode::BAD_GATEWAY)?, range)
    }

    async fn fence(&self, url: &Url) -> Result<AssetFence, StatusCode> {
        self.state
            .hls_sessions
            .asset_fence_at(&self.session, url, self.deadline)
            .await
            .map_err(bad_gateway)?
            .ok_or(StatusCode::NOT_FOUND)
    }

    async fn ensure_owner(&self, fence: &AssetFence) -> Result<(), StatusCode> {
        let owns = self
            .state
            .hls_sessions
            .owns_asset_at(&self.session, fence, self.deadline)
            .await
            .map_err(bad_gateway)?;
        owns.then_some(()).ok_or(StatusCode::BAD_GATEWAY)
    }

    async fn playback_object(&self, url: &Url) -> Result<Option<CachedHlsObject>, StatusCode> {
        let binding = self
            .state
            .hls_sessions
            .playback_binding(&self.session)
            .await
            .ok_or(StatusCode::NOT_FOUND)?;
        Ok(match binding {
            HlsPlaybackBinding::Prepared(asset) => asset.object(url.as_str()),
            HlsPlaybackBinding::Unprepared(_) => self.state.segmented.object(url.as_str()),
        })
    }
}

pub(super) fn bad_gateway(_: impl core::fmt::Display) -> StatusCode {
    StatusCode::BAD_GATEWAY
}
