#![allow(dead_code)]

pub fn tail_indexed_mp4() -> Vec<u8> {
    let file_type = atom(b"ftyp", joined(&[b"isom".to_vec(), vec![0; 4]]));
    let media = atom(b"mdat", b"DATA".to_vec());
    let sample_offset = (file_type.len() + 8) as u32;
    [file_type, media, classic_movie(sample_offset)].concat()
}

pub fn front_indexed_mp4() -> Vec<u8> {
    let file_type = atom(b"ftyp", joined(&[b"isom".to_vec(), vec![0; 4]]));
    let placeholder = classic_movie(0);
    let sample_offset = (file_type.len() + placeholder.len() + 8) as u32;
    [
        file_type,
        classic_movie(sample_offset),
        atom(b"mdat", b"DATA".to_vec()),
    ]
    .concat()
}

pub fn top_level_boxes(bytes: &[u8]) -> Vec<[u8; 4]> {
    let mut boxes = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        boxes.push(bytes[cursor + 4..cursor + 8].try_into().unwrap());
        cursor += u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
    }
    boxes
}

fn classic_movie(offset: u32) -> Vec<u8> {
    let stts = full_box(b"stts", values(&[1, 1, 1_000]));
    let stsc = full_box(b"stsc", values(&[1, 1, 1, 1]));
    let stsz = full_box(b"stsz", values(&[0, 1, 4]));
    let stco = full_box(b"stco", values(&[1, offset]));
    let stbl = atom(b"stbl", joined(&[stts, stsc, stsz, stco]));
    let minf = atom(b"minf", stbl);
    let mut mdhd = vec![0; 8];
    mdhd.extend(values(&[1_000, 1_000]));
    mdhd.extend([0; 4]);
    let mdhd = full_box(b"mdhd", mdhd);
    let mut handler = vec![0; 4];
    handler.extend(b"vide");
    let hdlr = full_box(b"hdlr", handler);
    let mdia = atom(b"mdia", joined(&[mdhd, hdlr, minf]));
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

fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.iter().flatten().copied().collect()
}
