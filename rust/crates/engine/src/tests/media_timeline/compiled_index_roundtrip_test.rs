use crate::media_timeline::compiled;
use crate::tests::media_timeline_dependency_support::{tail_timeline, with_sample_table};
use crate::tests::media_timeline_support::{full_box, values};

#[test]
fn compiled_index_preserves_composition_and_random_access_dependencies() {
    let mut ctts = full_box(
        b"ctts",
        values(&[3, 1, 1_000, 1, (-1_000_i32) as u32, 1, 0]),
    );
    ctts[8] = 1;
    let movie = with_sample_table(&[100, 200, 300], ctts);
    let timeline = tail_timeline(&movie);
    let total = 10_000 + movie.len() as u64;

    let encoded = compiled::encode(&timeline).expect("bounded compiled index");
    let restored = compiled::decode(&encoded, total).expect("valid compiled index");

    assert_eq!(restored, timeline);
    let record: serde_json::Value = serde_json::from_slice(&encoded).expect("fixture");
    assert_eq!(record["timeline"]["media"][1]["decode_start"], 1_000);
    assert_eq!(record["timeline"]["media"][1]["time"]["start"], 0);
    assert!(
        compiled::decode(&encoded, 200).is_err(),
        "wrong source extent must fail"
    );
}
