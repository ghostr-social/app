use anyhow::{Context as _, Result};
use ghostr_engine::RequestAuthority;
use reqwest::header::{HeaderMap, AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION};
use reqwest::{Method, Request, Url};

pub(super) struct ForwardedRequest {
    url: Url,
    method: Method,
    headers: HeaderMap,
    authority: RequestAuthority,
}

impl ForwardedRequest {
    pub(super) fn capture(request: &Request) -> Result<Self> {
        let authority = RequestAuthority::from_url(request.url().as_str())
            .context("executed media request authority is invalid")?;
        Ok(Self {
            url: request.url().clone(),
            method: request.method().clone(),
            headers: request.headers().clone(),
            authority,
        })
    }

    pub(super) fn url(&self) -> &Url {
        &self.url
    }

    pub(super) fn selection(
        &self,
        headers: &HeaderMap,
    ) -> super::super::response::selection::ResponseSelection {
        super::super::response::selection::ResponseSelection::capture(&self.headers, headers)
    }

    pub(super) fn redirected(mut self, target: &Url) -> Result<Self> {
        let authority =
            RequestAuthority::from_url(target.as_str()).context("redirect authority is invalid")?;
        if authority != self.authority {
            strip_sensitive(&mut self.headers);
        }
        self.url = target.clone();
        self.authority = authority;
        Ok(self)
    }

    pub(super) fn into_parts(self) -> (Method, HeaderMap) {
        (self.method, self.headers)
    }
}

fn strip_sensitive(headers: &mut HeaderMap) {
    headers.remove(AUTHORIZATION);
    headers.remove(COOKIE);
    headers.remove(PROXY_AUTHORIZATION);
}
