use super::{bad_gateway, AssetCall};
use crate::hls::asset_generation::{AssetFence, FirstAdmission, OriginGeneration};
use crate::hls::asset_request::AssetRangeRequest;
use crate::hls::asset_response::validate;
use crate::hls::transfer::HlsTransfer;
use axum::body::Body;
use axum::http::header::{ACCEPT_ENCODING, IF_RANGE};
use axum::http::{HeaderValue, Response, StatusCode};
use reqwest::{RequestBuilder, Url};

impl AssetCall {
    pub(super) async fn first(
        &self,
        url: Url,
        range: AssetRangeRequest,
        fence: AssetFence,
        admission: FirstAdmission,
    ) -> Result<Response<Body>, StatusCode> {
        let transfer = self.open(&url, range, None).await?;
        let envelope = validate(range, transfer.response()).map_err(bad_gateway)?;
        self.ensure_owner(&fence).await?;
        admission.admit(envelope, transfer.response());
        transfer.into_proxy(envelope)
    }

    pub(super) async fn continue_origin(
        &self,
        url: Url,
        range: AssetRangeRequest,
        fence: AssetFence,
        generation: OriginGeneration,
    ) -> Result<Response<Body>, StatusCode> {
        let transfer = self.open(&url, range, Some(generation.if_range())).await?;
        let envelope = match validate(range, transfer.response()) {
            Ok(envelope) => envelope,
            Err(_) => return self.reject_envelope(fence, generation, transfer).await,
        };
        fence
            .confirm_origin(&generation, envelope, transfer.response(), self.deadline)
            .await
            .map_err(bad_gateway)?;
        self.ensure_owner(&fence).await?;
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

    fn request(&self, url: &Url, range: AssetRangeRequest) -> Result<RequestBuilder, StatusCode> {
        let request = self
            .state
            .client
            .get(url.as_str())
            .map_err(bad_gateway)?
            .header(ACCEPT_ENCODING, "identity");
        Ok(range.apply(request))
    }
}
