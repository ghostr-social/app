use super::media_timeline_support::{classic_mdat_prefix, classic_moov, valid_ftyp};
use crate::media_timeline::{parse_mp4_segments, MediaSegment, MediaTimeline};

pub(super) fn with_sample_table(offsets: &[u32], table: Vec<u8>) -> Vec<u8> {
    insert_atom(
        classic_moov(offsets, &vec![100; offsets.len()]),
        &[b"moov", b"trak", b"mdia", b"minf", b"stbl"],
        table,
    )
}

pub(super) fn with_track_atom(table: Vec<u8>) -> Vec<u8> {
    insert_atom(
        classic_moov(&[100, 200, 300], &[100; 3]),
        &[b"moov", b"trak"],
        table,
    )
}

fn insert_atom(mut movie: Vec<u8>, kinds: &[&[u8; 4]], table: Vec<u8>) -> Vec<u8> {
    let mut insertion = 0;
    for kind in kinds {
        let at = movie
            .windows(4)
            .position(|bytes| bytes == *kind)
            .expect("fixture")
            - 4;
        let size = u32::from_be_bytes(movie[at..at + 4].try_into().expect("fixture"));
        let updated = size + table.len() as u32;
        movie[at..at + 4].copy_from_slice(&updated.to_be_bytes());
        insertion = at + size as usize;
    }
    movie.splice(insertion..insertion, table);
    movie
}

pub(super) fn try_tail_timeline(
    movie: &[u8],
) -> Result<MediaTimeline, crate::media_timeline::TimelineError> {
    let prefix = classic_mdat_prefix(&valid_ftyp(), 10_000, 500);
    parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(10_000, movie),
    ])
}

pub(super) fn tail_timeline(movie: &[u8]) -> MediaTimeline {
    try_tail_timeline(movie).expect("well-formed dependency fixture")
}
