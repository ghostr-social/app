use super::origin_concurrency;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;

const URL: &str = "https://media.example/video.mp4";

struct LocalClient(Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

#[tokio::test]
async fn predicted_origin_concurrency_is_the_next_slot_capped_by_authority_limit() {
    let requests = executor();
    let authority = RequestAuthority::from_url(URL).expect("valid test fixture");
    assert_eq!(origin_concurrency(&requests, &authority), 1);
    let first = admit(&requests, PreemptionAuthority::Transition).await;
    assert_eq!(origin_concurrency(&requests, &authority), 2);
    let second = admit(&requests, PreemptionAuthority::PlaybackCritical).await;
    assert_eq!(origin_concurrency(&requests, &authority), 2);
    drop((first, second));
}

async fn admit(
    requests: &MediaRequestExecutor,
    priority: PreemptionAuthority,
) -> ghostr_net::media_request_executor::AdmittedMediaRequest {
    requests
        .get(URL, priority)
        .expect("valid test fixture")
        .admit_for(Duration::from_secs(1))
        .await
        .expect("valid test fixture")
}

fn executor() -> MediaRequestExecutor {
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("valid test fixture");
    MediaRequestExecutor::new(
        Arc::new(LocalClient(client)),
        MediaRequestLimits::try_new(2, 2).expect("valid test fixture"),
    )
}
