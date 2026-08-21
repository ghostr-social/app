use super::{bad_gateway, AssetCall};
use crate::hls::asset_generation::{
    AssetFence, FirstAdmission, OriginConfirmation, OriginGeneration,
};
use crate::hls::asset_request::AssetRangeRequest;
use crate::hls::asset_response::validate;
use crate::hls::transfer::HlsTransfer;
use axum::body::Body;
use axum::http::header::{ACCEPT_ENCODING, IF_RANGE};
use axum::http::{HeaderValue, Response, StatusCode};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::MediaRequest;
use reqwest::Url;

pub(super) struct OriginRequest {
    pub url: Url,
    pub range: AssetRangeRequest,
    pub fence: AssetFence,
}

impl AssetCall {
    pub(super) async fn first(
        &self,
        request: OriginRequest,
        admission: FirstAdmission,
    ) -> Result<Response<Body>, StatusCode> {
        let transfer = self.open(&request.url, request.range, None).await?;
        let envelope = validate(request.range, transfer.response()).map_err(bad_gateway)?;
        self.ensure_owner(&request.fence).await?;
        admission.admit(envelope, transfer.response());
        transfer.into_proxy(envelope)
    }

    pub(super) async fn continue_origin(
        &self,
        request: OriginRequest,
        generation: OriginGeneration,
    ) -> Result<Response<Body>, StatusCode> {
        let transfer = self
            .open(&request.url, request.range, Some(generation.if_range()))
            .await?;
        let envelope = match validate(request.range, transfer.response()) {
            Ok(envelope) => envelope,
            Err(_) => {
                return self
                    .reject_envelope(request.fence, generation, transfer)
                    .await
            }
        };
        request
            .fence
            .confirm_origin(OriginConfirmation {
                expected: &generation,
                envelope,
                response: transfer.response(),
                deadline: self.deadline,
            })
            .await
            .map_err(bad_gateway)?;
        self.ensure_owner(&request.fence).await?;
        transfer.into_proxy(envelope)
    }

    async fn reject_envelope(
        &self,
        fence: AssetFence,
        generation: OriginGeneration,
        transfer: HlsTransfer,
    ) -> Result<Response<Body>, StatusCode> {
        if transfer.response().status().is_success() {
            fence
                .retire_origin(&generation, self.deadline)
                .await
                .map_err(bad_gateway)?;
        }
        Err(StatusCode::BAD_GATEWAY)
    }

    pub(super) async fn open(
        &self,
        url: &Url,
        range: AssetRangeRequest,
        if_range: Option<HeaderValue>,
    ) -> Result<HlsTransfer, StatusCode> {
        let mut request = self.request(url, range)?;
        if let Some(value) = if_range {
            request = request.header(IF_RANGE, value);
        }
        HlsTransfer::open_at(request, self.state.hls_timeouts, self.deadline)
            .await
            .map_err(bad_gateway)
    }

    fn request(&self, url: &Url, range: AssetRangeRequest) -> Result<MediaRequest, StatusCode> {
        let request = self
            .state
            .requests
            .get(url.as_str(), PreemptionAuthority::PlaybackCritical)
            .map_err(bad_gateway)?
            .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
        range.apply(request).map_err(bad_gateway)
    }
}
