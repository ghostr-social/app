use super::gate::{MediaRequestGate, RequestLease};
use super::request::{validate_request, MediaRequestAdmissionTimeout};
use super::response::MediaResponse;
use crate::outbound_media_client::MediaHttpRequests;
use crate::public_media_address::validate_url;
use anyhow::{ensure, Context as _, Result};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use reqwest::{Client, Request, Response, Url};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::Instant;

mod forwarded;
mod target;
use forwarded::ForwardedRequest;
use target::{redirect_target, visit_key};

const MAX_REDIRECTS: usize = 10;

pub(super) struct RedirectContext {
    client: Arc<dyn MediaHttpRequests>,
    gate: MediaRequestGate,
    priority: PreemptionAuthority,
    public_redirects_only: bool,
    maximum_body: u64,
}

#[derive(Clone, Copy)]
pub(super) struct RedirectPolicy {
    pub(super) public_only: bool,
    pub(super) maximum_body: u64,
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
        policy: RedirectPolicy,
    ) -> Self {
        Self {
            client,
            gate,
            priority,
            public_redirects_only: policy.public_only,
            maximum_body: policy.maximum_body,
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
        let mut lease = self.acquire(authority, deadline).await?;
        lease.reserve_body(self.maximum_body)?;
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

    async fn execute(mut self, deadline: Option<Instant>) -> Result<HopResponse> {
        if deadline.is_some_and(|value| Instant::now() >= value) {
            return Err(MediaRequestAdmissionTimeout.into());
        }
        let request = ForwardedRequest::capture(&self.request)?;
        self.lease.record_request();
        self.lease.sending();
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
        let mut result = hop.execute(deadline).await?;
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
        if result.response.content_length() == Some(0) {
            result.lease.complete_body()?;
        }
        drop(result.response);
        drop(result.lease);
        let (next, waited) = context.admit(target, request, deadline).await?;
        admission_wait = admission_wait.saturating_add(waited);
        hop = next;
    }
    unreachable!("redirect loop is bounded")
}
