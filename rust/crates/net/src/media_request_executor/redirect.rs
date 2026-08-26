use super::gate::{MediaRequestGate, RequestLease};
use super::request::{validate_request, MediaRequestAdmissionTimeout};
use super::response::MediaResponse;
use crate::outbound_media_client::MediaHttpRequests;
use crate::public_media_address::validate_url;
use anyhow::{ensure, Context as _, Result};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use reqwest::header::LOCATION;
use reqwest::{Client, Request, Response, StatusCode, Url};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::Instant;

mod forwarded;
use forwarded::ForwardedRequest;

const MAX_REDIRECTS: usize = 10;

pub(super) struct RedirectContext {
    client: Arc<dyn MediaHttpRequests>,
    gate: MediaRequestGate,
    priority: PreemptionAuthority,
    public_redirects_only: bool,
}

pub(super) struct AdmittedHop {
    client: Client,
    request: Request,
    lease: RequestLease,
}

struct HopResponse {
    response: Response,
    lease: RequestLease,
    request: ForwardedRequest,
}

impl RedirectContext {
    pub(super) fn new(
        client: Arc<dyn MediaHttpRequests>,
        gate: MediaRequestGate,
        priority: PreemptionAuthority,
        public_redirects_only: bool,
    ) -> Self {
        Self {
            client,
            gate,
            priority,
            public_redirects_only,
        }
    }

    async fn admit(
        &self,
        target: Url,
        request: ForwardedRequest,
        deadline: Option<Instant>,
    ) -> Result<(AdmittedHop, Duration)> {
        if self.public_redirects_only {
            validate_url(&target).context("media redirect target is not public")?;
        }
        let authority =
            RequestAuthority::from_url(target.as_str()).context("redirect authority is invalid")?;
        let (method, headers) = request.into_parts();
        let builder = self.client.get(target.as_str())?.headers(headers);
        let (client, next) = builder.build_split();
        let mut next = next.context("build redirected media request")?;
        validate_request(&next, &authority)?;
        *next.method_mut() = method;
        let started = Instant::now();
        let lease = self.acquire(authority, deadline).await?;
        Ok((AdmittedHop::new(client, next, lease), started.elapsed()))
    }

    async fn acquire(
        &self,
        authority: RequestAuthority,
        deadline: Option<Instant>,
    ) -> Result<RequestLease> {
        let acquiring = self.gate.acquire(authority, self.priority);
        let Some(deadline) = deadline else {
            return acquiring.await;
        };
        if Instant::now() >= deadline {
            return Err(MediaRequestAdmissionTimeout.into());
        }
        match tokio::time::timeout_at(deadline, acquiring).await {
            Ok(result) => result,
            Err(_) => Err(MediaRequestAdmissionTimeout.into()),
        }
    }
}

impl AdmittedHop {
    pub(super) fn new(client: Client, request: Request, lease: RequestLease) -> Self {
        Self {
            client,
            request,
            lease,
        }
    }

    async fn execute(self, deadline: Option<Instant>) -> Result<HopResponse> {
        if deadline.is_some_and(|value| Instant::now() >= value) {
            return Err(MediaRequestAdmissionTimeout.into());
        }
        let request = ForwardedRequest::capture(&self.request)?;
        self.lease.record_request();
        let response = self
            .client
            .execute(self.request)
            .await
            .context("send media request")?;
        ensure!(
            response.url() == request.url(),
            "media client followed a redirect internally"
        );
        Ok(HopResponse {
            response,
            lease: self.lease,
            request,
        })
    }
}

pub(super) async fn send(
    mut hop: AdmittedHop,
    context: RedirectContext,
    deadline: Option<Instant>,
) -> Result<MediaResponse> {
    let mut visited = HashSet::new();
    let mut admission_wait = Duration::ZERO;
    for followed in 0..=MAX_REDIRECTS {
        let result = hop.execute(deadline).await?;
        crate::response_limits::validate_response_headers(result.response.headers())
            .context("validate media response headers")?;
        visited.insert(visit_key(result.request.url()));
        let Some(target) = redirect_target(&result.response)? else {
            return Ok(MediaResponse::new(
                result.response,
                result.lease,
                admission_wait,
            ));
        };
        ensure!(followed < MAX_REDIRECTS, "media redirect limit exceeded");
        ensure!(
            visited.insert(visit_key(&target)),
            "media redirect loop detected"
        );
        let request = result.request.redirected(&target)?;
        drop(result.response);
        drop(result.lease);
        let (next, waited) = context.admit(target, request, deadline).await?;
        admission_wait = admission_wait.saturating_add(waited);
        hop = next;
    }
    unreachable!("redirect loop is bounded")
}

fn visit_key(url: &Url) -> Url {
    let mut key = url.clone();
    key.set_fragment(None);
    key
}

fn redirect_target(response: &Response) -> Result<Option<Url>> {
    if !followed_status(response.status()) {
        return Ok(None);
    }
    let Some(location) = response.headers().get(LOCATION) else {
        return Ok(None);
    };
    let location = location.to_str().context("redirect Location is not text")?;
    let target = response
        .url()
        .join(location)
        .context("redirect Location is invalid")?;
    ensure!(
        target.username().is_empty() && target.password().is_none(),
        "media redirect credentials are forbidden"
    );
    Ok(Some(target))
}

fn followed_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}
