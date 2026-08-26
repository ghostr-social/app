pub(super) fn cmaf_sidx(sizes: &[u32], durations: &[u32], first_offset: u32) -> Vec<u8> {
    assert_eq!(sizes.len(), durations.len());
    let mut body = values(&[1, 1_000, 0, first_offset]);
    body.extend([0_u8; 2]);
    body.extend((sizes.len() as u16).to_be_bytes());
    for (size, duration) in sizes.iter().zip(durations) {
        body.extend(size.to_be_bytes());
        body.extend(duration.to_be_bytes());
        body.extend(0_u32.to_be_bytes());
    }
    full_box(b"sidx", 0, body)
}

pub(super) fn cmaf_sidx_v1(timescale: u32, earliest: u64, size: u32, duration: u32) -> Vec<u8> {
    let mut body = values(&[1, timescale]);
    body.extend(earliest.to_be_bytes());
    body.extend(0_u64.to_be_bytes());
    body.extend([0_u8; 2]);
    body.extend(1_u16.to_be_bytes());
    body.extend(size.to_be_bytes());
    body.extend(duration.to_be_bytes());
    body.extend(0_u32.to_be_bytes());
    full_box(b"sidx", 1, body)
}

fn full_box(kind: &[u8; 4], version: u8, body: Vec<u8>) -> Vec<u8> {
    let mut payload = vec![version, 0, 0, 0];
    payload.extend(body);
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
