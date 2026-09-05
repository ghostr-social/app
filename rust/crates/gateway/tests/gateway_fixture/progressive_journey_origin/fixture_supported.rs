//! Supported counterpart of the fixture: remove its zero-origin identity edit.
//! The real player fixture retains the edit and exercises complete-file fallback.
pub(super) fn without_identity_edit(mut bytes: Vec<u8>) -> Vec<u8> {
    let kind = bytes
        .windows(4)
        .position(|value| value == b"edts")
        .expect("fixture edit");
    let start = kind - 4;
    assert_eq!(
        &bytes[start..start + 8],
        b"\0\0\0$edts",
        "one fixture edit box"
    );
    assert_eq!(
        &bytes[start + 8..start + 16],
        b"\0\0\0\x1celst",
        "version-zero edit list"
    );
    assert_eq!(
        &bytes[start + 16..start + 24],
        b"\0\0\0\0\0\0\0\x01",
        "one edit"
    );
    assert_eq!(
        &bytes[start + 28..start + 36],
        b"\0\0\0\0\0\x01\0\0",
        "zero origin at normal rate"
    );
    bytes[kind..kind + 4].copy_from_slice(b"free");
    bytes
}
