use super::warp_backward_spare_room_fixture::admitted_on;

#[test]
fn global_spare_capacity_prepares_history_only_when_its_origin_has_room() {
    assert!(!admitted_on("https://origin.example/previous.mp4"));
    assert!(admitted_on("https://independent.example/previous.mp4"));
}
