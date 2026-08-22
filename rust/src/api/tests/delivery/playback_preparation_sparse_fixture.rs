use crate::api::tests::support::sized_meta;
use ghostr_engine::adaptive::{candidate_snapshot, CandidateEvidence, FeedOffset, ViewProbability};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment, StartupFootprint};
use ghostr_engine::{ByteRange, EngineParams, PostId, VideoMeta};

pub(super) struct SparseStartupFixture {
    pub(super) meta: VideoMeta,
    pub(super) total: u64,
    pub(super) writes: Vec<(u64, Vec<u8>)>,
    pub(super) startup: StartupFootprint,
}

pub(super) fn sparse_startup() -> SparseStartupFixture {
    let ftyp = atom(b"ftyp", joined(&[b"isom".to_vec(), vec![0; 4]]));
    let prefix = classic_mdat_prefix(&ftyp, 512, 36);
    let moov = classic_moov();
    let timeline =
        parse_mp4_segments(&[MediaSegment::new(0, &prefix), MediaSegment::new(512, &moov)])
            .unwrap();
    SparseStartupFixture {
        meta: sized_meta(1_024, 1_000),
        total: 1_024,
        writes: vec![(0, prefix), (512, moov)],
        startup: timeline.startup_footprint().unwrap(),
    }
}

fn classic_mdat_prefix(file_type: &[u8], movie_start: u32, present_end: usize) -> Vec<u8> {
    let mut prefix = file_type.to_vec();
    let mdat_size = movie_start - prefix.len() as u32;
    prefix.extend(mdat_size.to_be_bytes());
    prefix.extend(b"mdat");
    prefix.resize(present_end.max(prefix.len()), 9);
    prefix
}

pub(super) fn complete_startup(meta: &VideoMeta, total: u64) -> StartupFootprint {
    let post = PostId::new("complete-fixture");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta.clone());
    candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(1),
            view_probability: ViewProbability::new(1.0).unwrap(),
            present: vec![ByteRange::new(0, total)],
            stored_total: Some(total),
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: Vec::new(),
        },
    )
    .and_then(|candidate| candidate.startup)
    .expect("complete startup footprint")
}

fn classic_moov() -> Vec<u8> {
    let stts = full_box(b"stts", words(&[1, 1, 1_000]));
    let stsc = full_box(b"stsc", words(&[1, 1, 1, 1]));
    let stsz = full_box(b"stsz", words(&[0, 1, 4]));
    let stco = full_box(b"stco", words(&[1, 32]));
    let stbl = atom(b"stbl", joined(&[stts, stsc, stsz, stco]));
    let minf = atom(b"minf", stbl);
    let mut mdhd = vec![0; 8];
    mdhd.extend(words(&[1_000, 1_000]));
    mdhd.extend([0; 4]);
    let mdhd = full_box(b"mdhd", mdhd);
    let mut hdlr = vec![0; 4];
    hdlr.extend(b"vide");
    let hdlr = full_box(b"hdlr", hdlr);
    let mdia = atom(b"mdia", joined(&[mdhd, hdlr, minf]));
    atom(b"moov", atom(b"trak", mdia))
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

fn words(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}
