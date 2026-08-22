use super::redirect_execution_fixture::{executor, redirected};
use ghostr_engine::adaptive::PreemptionAuthority;

#[tokio::test]
async fn guarded_redirect_execution_rejects_credentials_before_following() {
    let target = "https://user:secret@93.184.216.34/media.mp4";
    let (start, origin) = redirected(target).await;

    let result = executor(&start)
        .get(&start, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await;

    origin.await.unwrap();
    let Err(error) = result else {
        panic!("credential redirect was followed");
    };
    assert!(
        format!("{error:#}").contains("credential"),
        "unexpected redirect error: {error:#}"
    );
}
