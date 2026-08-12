pub(super) fn classic_moov(offsets: &[u32], sizes: &[u32]) -> Vec<u8> {
    assert_eq!(offsets.len(), sizes.len());
    let count = offsets.len() as u32;
    let stts = full_box(b"stts", values(&[1, count, 1_000]));
    let stsc = full_box(b"stsc", values(&[1, 1, 1, 1]));
    let mut stsz_body = values(&[0, count]);
    stsz_body.extend(values(sizes));
    let stsz = full_box(b"stsz", stsz_body);
    let mut stco_body = values(&[count]);
    stco_body.extend(values(offsets));
    let stco = full_box(b"stco", stco_body);
    let stbl = atom(b"stbl", joined(&[stts, stsc, stsz, stco]));
    let minf = atom(b"minf", stbl);
    let mut mdhd_body = vec![0; 8];
    mdhd_body.extend(values(&[1_000, count * 1_000]));
    mdhd_body.extend([0_u8; 4]);
    let mdhd = full_box(b"mdhd", mdhd_body);
    let mdia = atom(b"mdia", joined(&[mdhd, minf]));
    atom(b"moov", atom(b"trak", mdia))
}

pub(super) fn advanced_moov(offset: u64, sample_count: u32, sample_size: u32) -> Vec<u8> {
    let stts = full_box(b"stts", values(&[1, sample_count, 1_000]));
    let stsc = full_box(b"stsc", values(&[1, 1, sample_count, 1]));
    let stsz = full_box(b"stsz", values(&[sample_size, sample_count]));
    let mut co64_body = values(&[1]);
    co64_body.extend(offset.to_be_bytes());
    let co64 = full_box(b"co64", co64_body);
    let stbl = atom(b"stbl", joined(&[stts, stsc, stsz, co64]));
    let minf = atom(b"minf", stbl);
    let mut mdhd_body = vec![0; 16];
    mdhd_body.extend(values(&[1_000]));
    mdhd_body.extend(u64::from(sample_count).saturating_mul(1_000).to_be_bytes());
    mdhd_body.extend([0_u8; 4]);
    let mdhd = full_box_version(b"mdhd", 1, mdhd_body);
    let mdia = atom(b"mdia", joined(&[mdhd, minf]));
    atom(b"moov", atom(b"trak", mdia))
}

pub(super) fn classic_from_tables(
    stts_body: Vec<u8>,
    stsc_body: Vec<u8>,
    stsz_body: Vec<u8>,
    offsets: Vec<u8>,
) -> Vec<u8> {
    let stbl = atom(
        b"stbl",
        joined(&[
            full_box(b"stts", stts_body),
            full_box(b"stsc", stsc_body),
            full_box(b"stsz", stsz_body),
            offsets,
        ]),
    );
    let minf = atom(b"minf", stbl);
    let mut mdhd_body = vec![0; 8];
    mdhd_body.extend(values(&[1_000, 2_000]));
    mdhd_body.extend([0_u8; 4]);
    let mdhd = full_box(b"mdhd", mdhd_body);
    let mdia = atom(b"mdia", joined(&[mdhd, minf]));
    atom(b"moov", atom(b"trak", mdia))
}

pub(super) fn full_box(kind: &[u8; 4], body: Vec<u8>) -> Vec<u8> {
    full_box_version(kind, 0, body)
}

pub(super) fn full_box_version(kind: &[u8; 4], version: u8, body: Vec<u8>) -> Vec<u8> {
    let mut payload = vec![version, 0, 0, 0];
    payload.extend(body);
    atom(kind, payload)
}

pub(super) fn atom(kind: &[u8; 4], payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    bytes.extend(kind);
    bytes.extend(payload);
    bytes
}

pub(super) fn values(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

pub(super) fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}
