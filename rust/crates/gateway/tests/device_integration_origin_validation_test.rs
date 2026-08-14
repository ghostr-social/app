#![cfg(all(feature = "device-integration", debug_assertions))]

use ghostr_gateway::device_integration::DeviceIntegrationMediaHttpClient;

#[test]
fn integration_origin_rejects_every_scope_widening() {
    let rejected = [
        "https://127.0.0.1:4040",
        "http://user@127.0.0.1:4040",
        "http://:secret@127.0.0.1:4040",
        "http://127.0.0.1:4040/video.mp4",
        "http://127.0.0.1:4040?token=secret",
        "http://127.0.0.1:4040#fragment",
        "http://127.0.0.1",
        "http://localhost:4040",
        "http://192.0.2.1:4040",
    ];

    for origin in rejected {
        assert!(
            DeviceIntegrationMediaHttpClient::new(origin).is_err(),
            "unexpectedly accepted {origin}"
        );
    }
}
