pub(super) fn move_moov_to_tail(bytes: Vec<u8>) -> Vec<u8> {
    let (start, size) = top_level_box(&bytes, b"moov");
    let mut moved = Vec::with_capacity(bytes.len());
    moved.extend_from_slice(&bytes[..start]);
    moved.extend_from_slice(&bytes[start + size..]);
    moved.extend_from_slice(&bytes[start..start + size]);
    adjust_stco(&mut moved, size as u32);
    moved
}

fn top_level_box(bytes: &[u8], wanted: &[u8; 4]) -> (usize, usize) {
    let mut start = 0;
    while start + 8 <= bytes.len() {
        let size = read_u32(bytes, start) as usize;
        assert!(size >= 8 && start + size <= bytes.len(), "valid MP4 box");
        if &bytes[start + 4..start + 8] == wanted {
            return (start, size);
        }
        start += size;
    }
    panic!("missing MP4 box")
}

fn adjust_stco(bytes: &mut [u8], shift: u32) {
    let Some(kind) = bytes.windows(4).position(|window| window == b"stco") else {
        return;
    };
    let count = read_u32(bytes, kind + 8) as usize;
    for index in 0..count {
        let offset = kind + 12 + index * 4;
        let value = read_u32(bytes, offset);
        write_u32(
            bytes,
            offset,
            value.checked_sub(shift).expect("media offset"),
        );
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().expect("u32 bytes"))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}
