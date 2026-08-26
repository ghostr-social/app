use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;

struct LocalMediaClient(Client);

impl MediaHttpRequests for LocalMediaClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub fn media_client() -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        raw_media_client(),
        MediaRequestLimits::try_new(4, 4).expect("valid test fixture"),
    )
}

pub fn raw_media_client() -> Arc<dyn MediaHttpRequests> {
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("local media client");
    Arc::new(LocalMediaClient(client))
}
