//! Failure classification over the errors the media stack really
//! produces: a host that cannot resolve, a rejected URL, and a 404 are
//! permanent-ish; a 5xx is worth another try.

mod range_fixture;

use anyhow::{Context as _, Result};
use ghostr_delivery::manager::failure::{classify, FailureClass};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpRequests};

const UNRESOLVABLE: &str = "http://cdn.ghostr-nonexistent.invalid/video.mp4";

async fn attempt(client: &dyn MediaHttpRequests, url: &str) -> Result<()> {
    let response = client
        .get(url)?
        .send()
        .await
        .context("chunk request failed")?;
    response
        .error_for_status()
        .context("chunk request rejected")?;
    Ok(())
}

async fn class_of(client: &dyn MediaHttpRequests, url: &str) -> FailureClass {
    let error = attempt(client, url)
        .await
        .expect_err("the fixture must reject this request");
    classify(&error)
}

#[tokio::test]
async fn delivery_failure_classes_follow_the_error_kind() {
    let trusted = range_fixture::raw_media_client();
    let missing = range_fixture::ranged::serve_ranged(range_fixture::body())
        .await
        .replace("/video.mp4", "/gone.mp4");
    let broken = range_fixture::reject::serve_failing().await;

    let cases = [
        (UNRESOLVABLE.to_owned(), FailureClass::Permanent),
        (missing, FailureClass::Permanent),
        (broken, FailureClass::Transient),
    ];

    for (url, expected) in cases {
        assert_eq!(class_of(trusted.as_ref(), &url).await, expected, "{url}");
    }
}

#[tokio::test]
async fn delivery_failure_treats_a_blocked_destination_as_permanent() {
    let guarded = MediaHttpClient::public().expect("public media client");

    let class = class_of(&guarded, "http://127.0.0.1:9/video.mp4").await;

    assert_eq!(class, FailureClass::Permanent);
}
