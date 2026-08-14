const SCALARS: [(&[u8; 4], usize, bool); 5] = [
    (b"mvhd", 24, false),
    (b"tkhd", 28, false),
    (b"elst", 16, false),
    (b"elst", 20, true),
    (b"mdhd", 24, false),
];

pub(super) fn scale_avc_timing(mut bytes: Vec<u8>, multiplier: u32) -> Vec<u8> {
    assert!(multiplier > 0, "positive timing multiplier");
    for (box_type, offset, signed) in SCALARS {
        scale_scalar(&mut bytes, box_type, offset, signed, multiplier);
    }
    for box_type in [b"stts", b"ctts"] {
        scale_table(&mut bytes, box_type, multiplier);
    }
    bytes
}

fn scale_scalar(bytes: &mut [u8], box_type: &[u8], field: usize, signed: bool, factor: u32) {
    let offset = box_start(bytes, box_type) + field;
    if signed {
        let value = read_u32(bytes, offset) as i32;
        write_u32(bytes, offset, (value * factor as i32) as u32);
    } else {
        write_u32(bytes, offset, read_u32(bytes, offset) * factor);
    }
}

fn scale_table(bytes: &mut [u8], box_type: &[u8], factor: u32) {
    let start = box_start(bytes, box_type);
    let count = read_u32(bytes, start + 12) as usize;
    for index in 0..count {
        let offset = start + 20 + index * 8;
        write_u32(bytes, offset, read_u32(bytes, offset) * factor);
    }
}

fn box_start(bytes: &[u8], box_type: &[u8]) -> usize {
    bytes
        .windows(box_type.len())
        .position(|window| window == box_type)
        .expect("MP4 fixture box")
        - 4
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().expect("u32 bytes"))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}
