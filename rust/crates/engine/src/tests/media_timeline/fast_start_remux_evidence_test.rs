use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::cmaf_timeline_support::cmaf_sidx;
use crate::tests::media_timeline_support::{
    atom, classic_mdat_prefix, classic_moov, joined, valid_ftyp,
};

#[test]
fn only_a_fully_inspected_classic_tail_movie_is_fast_start_recoverable() {
    let tail = tail_movie();
    let front = front_movie();
    let fragmented = [valid_ftyp(), cmaf_sidx(&[4], &[1_000], 0)].concat();
    let moof_hybrid = tail_with_marker(b"moof");
    let mfra_hybrid = tail_with_marker(b"mfra");

    assert!(parse(&tail).fast_start_remuxable(tail.len() as u64));
    assert!(!parse(&front).fast_start_remuxable(front.len() as u64));
    assert!(!parse(&fragmented).fast_start_remuxable(fragmented.len() as u64));
    assert!(!parse(&moof_hybrid).fast_start_remuxable(moof_hybrid.len() as u64));
    assert!(!parse(&mfra_hybrid).fast_start_remuxable(mfra_hybrid.len() as u64));
    assert!(parse_mp4_segments(&[MediaSegment::new(0, b"not an mp4")]).is_err());
}

#[test]
fn sparse_head_and_tail_metadata_cannot_claim_exact_remuxability() {
    let file_type = valid_ftyp();
    let movie = classic_moov(&[24], &[4]);
    let prefix = classic_mdat_prefix(&file_type, 512, 28);
    let total = 512 + movie.len() as u64;
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(512, &movie),
    ])
    .expect("valid test fixture");

    assert!(timeline.startup_footprint().is_some());
    assert!(!timeline.fast_start_remuxable(total));
}

fn parse(bytes: &[u8]) -> crate::media_timeline::MediaTimeline {
    parse_mp4_segments(&[MediaSegment::new(0, bytes)]).expect("valid test fixture")
}

fn tail_movie() -> Vec<u8> {
    let file_type = valid_ftyp();
    let media = atom(b"mdat", b"DATA".to_vec());
    let movie = classic_moov(&[(file_type.len() + 8) as u32], &[4]);
    joined(&[file_type, media, movie])
}

fn tail_with_marker(kind: &[u8; 4]) -> Vec<u8> {
    let file_type = valid_ftyp();
    let media = atom(b"mdat", b"DATA".to_vec());
    let marker = atom(kind, Vec::new());
    let movie = classic_moov(&[(file_type.len() + 8) as u32], &[4]);
    joined(&[file_type, media, marker, movie])
}

fn front_movie() -> Vec<u8> {
    let file_type = valid_ftyp();
    let placeholder = classic_moov(&[0], &[4]);
    let offset = (file_type.len() + placeholder.len() + 8) as u32;
    let movie = classic_moov(&[offset], &[4]);
    joined(&[file_type, movie, atom(b"mdat", b"DATA".to_vec())])
}
