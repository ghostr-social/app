use super::gate::{MediaRequestGate, RequestLease};
use super::response::MediaResponse;
use anyhow::{ensure, Context, Result};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use reqwest::header::{HeaderName, HeaderValue, HOST};
use reqwest::{Client, Method, Request, RequestBuilder};

pub struct MediaRequest {
    builder: RequestBuilder,
    authority: RequestAuthority,
    priority: PreemptionAuthority,
    method: Option<Method>,
    gate: MediaRequestGate,
}

pub struct AdmittedMediaRequest {
    client: Client,
    request: Request,
    lease: RequestLease,
}

impl MediaRequest {
    pub(super) fn new(
        builder: RequestBuilder,
        authority: RequestAuthority,
        priority: PreemptionAuthority,
        gate: MediaRequestGate,
    ) -> Self {
        Self {
            builder,
            authority,
            priority,
            method: None,
            gate,
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
        validate_request(&request, &self.authority)?;
        *request.method_mut() = self.method.unwrap_or(Method::GET);
        let lease = self.gate.acquire(self.authority, self.priority).await?;
        Ok(AdmittedMediaRequest {
            client,
            request,
            lease,
        })
    }
}

fn validate_request(request: &Request, expected: &RequestAuthority) -> Result<()> {
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
        let response = self
            .client
            .execute(self.request)
            .await
            .context("send media request")?;
        Ok(MediaResponse::new(response, self.lease))
    }
}
