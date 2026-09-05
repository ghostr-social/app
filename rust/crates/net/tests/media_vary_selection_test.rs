mod media_selection_fixture;
mod request_gate_fixture;

use ghostr_net::media_retention::MediaRetention;
use media_selection_fixture::{headers, response};

#[tokio::test]
async fn varying_cookie_and_encoding_preserves_anonymous_range_selection() {
    let first = response("Cookie, accept-encoding", headers("bytes=0-0")).await;
    let second = response("Accept-Encoding, COOKIE", headers("bytes=1-1")).await;
    assert_eq!(first.retention(), MediaRetention::Partitioned);
    assert!(first.request_selection().is_some());
    assert_eq!(first.request_selection(), second.request_selection());
}

#[tokio::test]
async fn selected_request_fields_are_part_of_the_generation_identity() {
    let first = response("Accept-Language", headers("bytes=0-0")).await;
    let mut french = headers("bytes=0-0");
    french.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "fr".parse().expect("language"),
    );
    let second = response("Accept-Language", french).await;
    assert_ne!(first.request_selection(), second.request_selection());
}

#[tokio::test]
async fn vary_range_does_not_authorize_splicing_different_range_selections() {
    let first = response("Range", headers("bytes=0-0")).await;
    let second = response("Range", headers("bytes=1-1")).await;
    assert_ne!(first.request_selection(), second.request_selection());
}

#[tokio::test]
async fn wildcard_and_malformed_vary_remain_transient() {
    for vary in ["*", "Cookie, *", "bad field", "Cookie,,Range"] {
        let observed = response(vary, headers("bytes=0-0")).await;
        assert_eq!(observed.retention(), MediaRetention::Transient, "{vary}");
        assert!(observed.request_selection().is_none());
    }
}

#[tokio::test]
async fn credentials_are_not_promoted_to_reusable_anonymous_media() {
    let mut selected = headers("bytes=0-0");
    selected.insert(
        reqwest::header::COOKIE,
        "session=secret".parse().expect("cookie"),
    );
    let observed = response("Cookie", selected).await;
    assert_eq!(observed.retention(), MediaRetention::Transient);
}

#[tokio::test]
async fn selecting_a_variant_never_overrides_no_store_or_set_cookie() {
    for policy in ["Cache-Control: no-store", "Set-Cookie: session=secret"] {
        let observed = response(&format!("Cookie\r\n{policy}"), headers("bytes=0-0")).await;
        assert_eq!(observed.retention(), MediaRetention::Transient);
    }
}
