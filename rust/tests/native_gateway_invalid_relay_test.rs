mod support;

use rust_lib_ghostr::video::video::ffi_start_server;
use support::fixtures::temp_directory;

#[tokio::test]
async fn ignores_a_rejected_relay_while_starting_the_local_gateway() {
    let directory = temp_directory("ghostr-gateway-relay");

    let result = ffi_start_server(
        directory.to_string_lossy().to_string(),
        1,
        1024,
        "  \n%%%\n  ".to_owned(),
    )
    .await;

    assert!(result.is_ok());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
