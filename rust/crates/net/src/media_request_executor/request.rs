use super::gate::MediaRequestGate;
use super::redirect::{self, AdmittedHop, RedirectContext};
use super::response::MediaResponse;
use crate::outbound_media_client::MediaHttpRequests;
use crate::public_media_address::validate_url;
use anyhow::{ensure, Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use reqwest::header::{HeaderName, HeaderValue, HOST};
use reqwest::{Method, Request, RequestBuilder, Url};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Debug)]
pub struct MediaRequestAdmissionTimeout;

impl fmt::Display for MediaRequestAdmissionTimeout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("media request admission timed out")
    }
}

impl std::error::Error for MediaRequestAdmissionTimeout {}

pub struct MediaRequest {
    builder: RequestBuilder,
    route: RequestRoute,
    method: Option<Method>,
}

pub struct AdmittedMediaRequest {
    hop: AdmittedHop,
    redirects: RedirectContext,
}

pub(super) struct RequestRoute {
    client: Arc<dyn MediaHttpRequests>,
    gate: MediaRequestGate,
    authority: RequestAuthority,
    priority: PreemptionAuthority,
    public_redirects_only: bool,
}

impl RequestRoute {
    pub(super) fn new(
        client: Arc<dyn MediaHttpRequests>,
        gate: MediaRequestGate,
        raw_url: &str,
        priority: PreemptionAuthority,
    ) -> Result<Self> {
        let url = Url::parse(raw_url).context("media request URL is invalid")?;
        let authority =
            RequestAuthority::from_url(raw_url).context("media request authority is invalid")?;
        Ok(Self {
            client,
            gate,
            authority,
            priority,
            public_redirects_only: validate_url(&url).is_ok(),
        })
    }
}

impl MediaRequest {
    pub(super) fn new(builder: RequestBuilder, route: RequestRoute) -> Self {
        Self {
            builder,
            route,
            method: None,
        }
    }

    pub fn header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.builder = self.builder.header(name, value);
        self
    }

    pub fn head(mut self) -> Self {
        self.method = Some(Method::HEAD);
        self
    }

    pub async fn admit(self) -> Result<AdmittedMediaRequest> {
        let (client, request) = self.builder.build_split();
        let mut request = request.context("build media request")?;
        validate_request(&request, &self.route.authority)?;
        *request.method_mut() = self.method.unwrap_or(Method::GET);
        let lease = self
            .route
            .gate
            .acquire(self.route.authority, self.route.priority)
            .await?;
        Ok(AdmittedMediaRequest {
            hop: AdmittedHop::new(client, request, lease),
            redirects: RedirectContext::new(
                self.route.client,
                self.route.gate,
                self.route.priority,
                self.route.public_redirects_only,
            ),
        })
    }

    pub async fn admit_for(self, wait: Duration) -> Result<AdmittedMediaRequest> {
        match tokio::time::timeout(wait, self.admit()).await {
            Ok(result) => result,
            Err(_) => Err(MediaRequestAdmissionTimeout.into()),
        }
    }
}

pub(super) fn validate_request(request: &Request, expected: &RequestAuthority) -> Result<()> {
    let actual = RequestAuthority::from_url(request.url().as_str())
        .context("built media request authority is invalid")?;
    ensure!(actual == *expected, "media request authority was rewritten");
    ensure!(
        !request.headers().contains_key(HOST),
        "explicit Host is forbidden"
    );
    ensure!(request.body().is_none(), "media request body is forbidden");
    Ok(())
}

impl AdmittedMediaRequest {
    pub async fn send(self) -> Result<MediaResponse> {
        redirect::send(self.hop, self.redirects, None).await
    }

    /// Sends while giving every redirect admission the caller's absolute deadline.
    /// The caller remains responsible for applying the same deadline to origin IO.
    pub async fn send_with_redirect_deadline(self, deadline: Instant) -> Result<MediaResponse> {
        redirect::send(self.hop, self.redirects, Some(deadline)).await
    }
}
