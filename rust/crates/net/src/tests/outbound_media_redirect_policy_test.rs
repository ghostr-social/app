use super::redirect_execution_fixture::{executor, redirected};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use tokio::net::TcpListener;

#[tokio::test]
async fn guarded_redirect_execution_rejects_a_private_target() {
    let target = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let target_address = target.local_addr().expect("valid test fixture");
    let (start, origin) = redirected(&format!("http://{target_address}/private")).await;
    let target_hit = tokio::spawn(async move { target.accept().await });

    let result = executor(&start)
        .get(&start, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await;

    assert!(result.is_err());
    origin.await.expect("valid test fixture");
    assert!(tokio::time::timeout(Duration::from_millis(75), target_hit)
        .await
        .is_err());
}
