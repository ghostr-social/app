use ghostr_delivery::delivery_events::{CommandReceiver, DeliveryCommand};
use std::time::Duration;

pub async fn next_control(commands: &mut CommandReceiver) -> DeliveryCommand {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(command) = commands.try_control() {
                break command;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("delivery control command")
}
