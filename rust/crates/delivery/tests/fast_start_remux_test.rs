use core::time::Duration;
use ghostr_delivery::transform::{
    FastStartRemuxBackend, TransformBackend as _, TransformControl, TransformInput,
};
use ghostr_engine::adaptive::TransformKind;
use std::time::Instant;

#[test]
fn remux_moves_moov_and_repairs_exact_chunk_offsets() {
    let input = tail_indexed_mp4();
    let control = TransformControl::new(Instant::now() + Duration::from_secs(1));
    let output = FastStartRemuxBackend::production()
        .transform(TransformInput::new(TransformKind::Remux, &input), &control)
        .expect("valid test fixture")
        .into_bytes();

    assert_eq!(boxes(&output), [*b"ftyp", *b"moov", *b"mdat"]);
    let stco = find_type(&output, b"stco");
    assert_eq!(
        u32::from_be_bytes(
            output[stco + 16..stco + 20]
                .try_into()
                .expect("valid test fixture")
        ),
        80
    );
    assert_eq!(output.len(), input.len());
}

fn tail_indexed_mp4() -> Vec<u8> {
    let mut offset = Vec::from([0, 0, 0, 0, 0, 0, 0, 1]);
    offset.extend_from_slice(&20_u32.to_be_bytes());
    let moov = container(
        *b"moov",
        container(
            *b"trak",
            container(
                *b"mdia",
                container(*b"minf", container(*b"stbl", atom(*b"stco", offset))),
            ),
        ),
    );
    [
        atom(*b"ftyp", b"isom".to_vec()),
        atom(*b"mdat", b"DATA".to_vec()),
        moov,
    ]
    .concat()
}

fn atom(kind: [u8; 4], payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::from(((payload.len() + 8) as u32).to_be_bytes());
    bytes.extend(kind);
    bytes.extend(payload);
    bytes
}

fn container(kind: [u8; 4], child: Vec<u8>) -> Vec<u8> {
    atom(kind, child)
}

fn boxes(bytes: &[u8]) -> Vec<[u8; 4]> {
    let mut kinds = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        kinds.push(
            bytes[cursor + 4..cursor + 8]
                .try_into()
                .expect("valid test fixture"),
        );
        cursor += u32::from_be_bytes(
            bytes[cursor..cursor + 4]
                .try_into()
                .expect("valid test fixture"),
        ) as usize;
    }
    kinds
}

fn find_type(bytes: &[u8], kind: &[u8; 4]) -> usize {
    bytes
        .windows(4)
        .position(|window| window == kind)
        .expect("valid test fixture")
        - 4
}
