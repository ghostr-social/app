use crate::RequestAuthority;

#[test]
fn authority_canonicalizes_http_origin_identity() {
    for (left, right) in [
        ("https://EXAMPLE.com/a", "https://example.com:443/b?q=1"),
        ("https://[2001:0db8::1]/a", "https://[2001:db8::1]:443/b"),
        (
            "https://bücher.example/a",
            "https://xn--bcher-kva.example/b",
        ),
    ] {
        assert_eq!(authority(left), authority(right));
    }
}

#[test]
fn authority_keeps_scheme_and_nondefault_port_distinct() {
    assert_ne!(
        authority("http://example.com/a"),
        authority("https://example.com/a")
    );
    assert_ne!(
        authority("https://example.com/a"),
        authority("https://example.com:8443/a")
    );
}

#[test]
fn authority_rejects_non_http_malformed_and_credentialed_urls() {
    for source in [
        "relative/path",
        "ftp://example.com/media",
        "https://",
        "https://user@example.com/media",
        "https://:secret@example.com/media",
    ] {
        assert!(RequestAuthority::from_url(source).is_none(), "{source}");
    }
}

fn authority(source: &str) -> RequestAuthority {
    RequestAuthority::from_url(source).expect("valid request authority")
}
