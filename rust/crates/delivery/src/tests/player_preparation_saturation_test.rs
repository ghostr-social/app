use crate::delivery_events::{command_channel, PlayerPreparationIngress};
use crate::tests::player_preparation_fixture::report;

#[test]
fn saturated_initial_never_enters_the_receiver() {
    let (handle, receiver) = command_channel();
    for index in 0..16 {
        let admission = handle.player_preparation_admission();
        assert_eq!(
            handle.report_player_preparation_initial(
                admission,
                report(&format!("post-{index}"), index + 1, index + 1),
            ),
            PlayerPreparationIngress::Accepted,
        );
    }
    let admission = handle.player_preparation_admission();
    assert_eq!(
        handle.report_player_preparation_initial(admission, report("overflow", 17, 17)),
        PlayerPreparationIngress::Saturated,
    );

    let mut received = Vec::new();
    while let Some(item) = receiver.try_player_preparation() {
        received.push(item);
    }
    assert_eq!(received.len(), 16);
    assert!(received
        .iter()
        .all(|item| item.post().as_str() != "overflow"));
}
