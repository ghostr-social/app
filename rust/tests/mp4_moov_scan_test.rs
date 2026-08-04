use rust_lib_ghostr::video::mp4_moov::head_contains_moov;

fn full_box(kind: &[u8; 4], payload_len: usize) -> Vec<u8> {
    let mut bytes = ((payload_len + 8) as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(kind);
    bytes.extend(std::iter::repeat_n(0u8, payload_len));
    bytes
}

fn sized_header(kind: &[u8; 4], declared: u32) -> Vec<u8> {
    let mut bytes = declared.to_be_bytes().to_vec();
    bytes.extend_from_slice(kind);
    bytes
}

fn large_box(kind: &[u8; 4], largesize: u64, payload_len: usize) -> Vec<u8> {
    let mut bytes = sized_header(kind, 1);
    bytes.extend_from_slice(&largesize.to_be_bytes());
    bytes.extend(std::iter::repeat_n(0u8, payload_len));
    bytes
}

fn joined(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

#[test]
fn detects_a_top_level_moov_box_within_the_available_head_bytes() {
    let ftyp = full_box(b"ftyp", 8);
    let moov = full_box(b"moov", 16);
    let cases: Vec<(Vec<u8>, bool)> = vec![
        (joined(&[ftyp.clone(), moov, full_box(b"mdat", 4)]), true),
        (joined(&[ftyp.clone(), sized_header(b"mdat", 1_000_000)]), false),
        (joined(&[ftyp.clone(), sized_header(b"moov", 1_000_000)]), true),
        (joined(&[ftyp.clone(), sized_header(b"mdat", 0)]), false),
        (joined(&[ftyp.clone(), sized_header(b"moov", 0)]), true),
        (
            joined(&[large_box(b"free", 24, 8), full_box(b"moov", 4)]),
            true,
        ),
        (
            joined(&[large_box(b"free", 8, 8), full_box(b"moov", 4)]),
            false,
        ),
        (joined(&[sized_header(b"free", 1), full_box(b"moov", 4)]), false),
        (joined(&[sized_header(b"free", 4), full_box(b"moov", 4)]), false),
        (ftyp[..6].to_vec(), false),
        (Vec::new(), false),
    ];
    for (index, (head, expected)) in cases.into_iter().enumerate() {
        assert_eq!(head_contains_moov(&head), expected, "case {index}");
    }
}
