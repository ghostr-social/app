use crate::manager::state::DeliveryState;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::PostId;

pub(super) fn classic_moov(offset: u32, size: u32) -> Vec<u8> {
    classic_samples(&[offset], &[size])
}

pub(super) fn classic_samples(offsets: &[u32], sizes: &[u32]) -> Vec<u8> {
    assert_eq!(offsets.len(), sizes.len());
    let count = offsets.len() as u32;
    let stts = full_box(b"stts", values(&[1, count, 1_000]));
    let stsc = full_box(b"stsc", values(&[1, 1, 1, 1]));
    let stsz = full_box(b"stsz", [values(&[0, count]), values(sizes)].concat());
    let stco = full_box(b"stco", [values(&[count]), values(offsets)].concat());
    let stbl = atom(b"stbl", joined(&[stts, stsc, stsz, stco]));
    let minf = atom(b"minf", stbl);
    let mut mdhd_body = vec![0; 8];
    mdhd_body.extend(values(&[1_000, count * 1_000]));
    mdhd_body.extend([0_u8; 4]);
    let mdhd = full_box(b"mdhd", mdhd_body);
    let mdia = atom(b"mdia", joined(&[mdhd, minf]));
    atom(b"moov", atom(b"trak", mdia))
}

pub(super) fn install_classic_timeline(
    state: &mut DeliveryState,
    post: &PostId,
    offset: u32,
    size: u32,
) {
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &classic_moov(offset, size))])
        .expect("classic fixture timeline");
    let binding = state.catalog().binding(post).expect("fixture binding");
    assert!(state.catalog_mut().learn_timeline_for(&binding, timeline));
}

fn full_box(kind: &[u8; 4], body: Vec<u8>) -> Vec<u8> {
    let mut payload = vec![0; 4];
    payload.extend(body);
    atom(kind, payload)
}

fn atom(kind: &[u8; 4], payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    bytes.extend(kind);
    bytes.extend(payload);
    bytes
}

fn values(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}
