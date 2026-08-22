use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(super) fn store(prefix: &str) -> (PathBuf, PartialRangeStore) {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("ghostr-{prefix}-{}-{unique}", std::process::id()));
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    (root, store)
}

pub(super) fn binding() -> ghostr_engine::representation::RepresentationBinding {
    let post = ghostr_engine::PostId::new("next");
    let meta = ghostr_engine::VideoMeta {
        urls: vec!["https://media.example/next.mp4".into()],
        delivery: ghostr_engine::DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(216),
        duration_ms: Some(1_000),
    };
    ghostr_engine::catalog::Catalog::new().upsert(post, meta)
}

pub(super) fn tail_indexed_mp4() -> Vec<u8> {
    let file_type = atom(b"ftyp", [b"isom".as_slice(), &[0; 4]].concat());
    let media = atom(b"mdat", b"DATA".to_vec());
    let movie = movie((file_type.len() + 8) as u32);
    [file_type, media, movie].concat()
}

fn movie(offset: u32) -> Vec<u8> {
    let stts = full_box(b"stts", values(&[1, 1, 1_000]));
    let stsc = full_box(b"stsc", values(&[1, 1, 1, 1]));
    let stsz = full_box(b"stsz", values(&[0, 1, 4]));
    let stco = full_box(b"stco", values(&[1, offset]));
    let stbl = atom(b"stbl", [stts, stsc, stsz, stco].concat());
    let mut mdhd = vec![0; 8];
    mdhd.extend(values(&[1_000, 1_000]));
    mdhd.extend([0; 4]);
    let mut handler = vec![0; 4];
    handler.extend(b"vide");
    let mdia = atom(
        b"mdia",
        [
            full_box(b"mdhd", mdhd),
            full_box(b"hdlr", handler),
            atom(b"minf", stbl),
        ]
        .concat(),
    );
    atom(b"moov", atom(b"trak", mdia))
}

fn full_box(kind: &[u8; 4], body: Vec<u8>) -> Vec<u8> {
    atom(kind, [vec![0; 4], body].concat())
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
