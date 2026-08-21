use super::origin_concurrency;
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
    let authority = RequestAuthority::from_url(URL).unwrap();
    assert_eq!(origin_concurrency(&requests, &authority), 1);
    let first = admit(&requests).await;
    assert_eq!(origin_concurrency(&requests, &authority), 2);
    let second = admit(&requests).await;
    assert_eq!(origin_concurrency(&requests, &authority), 2);
    drop((first, second));
}

async fn admit(
    requests: &MediaRequestExecutor,
) -> ghostr_net::media_request_executor::AdmittedMediaRequest {
    requests
        .get(URL, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
}

fn executor() -> MediaRequestExecutor {
    let client = Client::builder().no_proxy().build().unwrap();
    MediaRequestExecutor::new(
        Arc::new(LocalClient(client)),
        MediaRequestLimits::try_new(2, 2).unwrap(),
    )
}
