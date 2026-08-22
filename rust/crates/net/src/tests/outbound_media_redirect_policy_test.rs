use super::redirect_execution_fixture::{executor, redirected};
use ghostr_engine::adaptive::PreemptionAuthority;
use std::time::Duration;
use tokio::net::TcpListener;

#[tokio::test]
async fn guarded_redirect_execution_rejects_a_private_target() {
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_address = target.local_addr().unwrap();
    let (start, origin) = redirected(&format!("http://{target_address}/private")).await;
    let target_hit = tokio::spawn(async move { target.accept().await });

    let result = executor(&start)
        .get(&start, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await;

    assert!(result.is_err());
    origin.await.unwrap();
    assert!(tokio::time::timeout(Duration::from_millis(75), target_hit)
        .await
        .is_err());
}
