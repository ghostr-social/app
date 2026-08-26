use crate::transform::TransformControl;
use anyhow::{ensure, Context as _, Result};
use core::ops::Range;

#[derive(Clone, Copy)]
struct Atom {
    start: usize,
    end: usize,
    kind: [u8; 4],
}

pub(super) fn fast_start(input: &[u8], control: &TransformControl) -> Result<Vec<u8>> {
    control.checkpoint()?;
    let atoms = atoms(input, 0, input.len(), control)?;
    ensure!(
        atoms.first().is_some_and(|atom| atom.kind == *b"ftyp"),
        "MP4 has no leading ftyp atom"
    );
    let media = exactly_one(&atoms, *b"mdat")?;
    let index = exactly_one(&atoms, *b"moov")?;
    ensure!(
        media.end <= index.start && index.end == input.len(),
        "MP4 is not tail-indexed"
    );
    let mut patched = input[index.start..index.end].to_vec();
    let media_payload = media.start.saturating_add(8)..media.end;
    let delta = u64::try_from(patched.len()).context("moov atom exceeds offset space")?;
    let patched_offsets = patch_children(&mut patched, 8, media_payload, delta, control)?;
    ensure!(
        patched_offsets > 0,
        "MP4 moov has no supported chunk offsets"
    );
    assemble(input, media.start, index.start, &patched)
}

fn exactly_one(atoms: &[Atom], kind: [u8; 4]) -> Result<Atom> {
    let mut matches = atoms.iter().filter(|atom| atom.kind == kind);
    let atom = *matches.next().context("required MP4 atom is absent")?;
    ensure!(matches.next().is_none(), "MP4 has duplicate required atoms");
    Ok(atom)
}

fn assemble(input: &[u8], media: usize, index: usize, moov: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(input.len())
        .context("reserve remux output")?;
    output.extend_from_slice(&input[..media]);
    output.extend_from_slice(moov);
    output.extend_from_slice(&input[media..index]);
    ensure!(
        output.len() == input.len(),
        "remux changed representation length"
    );
    Ok(output)
}

fn atoms(bytes: &[u8], start: usize, end: usize, control: &TransformControl) -> Result<Vec<Atom>> {
    let mut cursor = start;
    let mut parsed = Vec::new();
    while cursor < end {
        control.checkpoint()?;
        ensure!(end.saturating_sub(cursor) >= 8, "truncated MP4 atom header");
        let size = read_u32(bytes, cursor)? as usize;
        ensure!(size >= 8, "unsupported MP4 atom size");
        let atom_end = cursor.checked_add(size).context("MP4 atom size overflow")?;
        ensure!(atom_end <= end, "MP4 atom exceeds its container");
        parsed.push(Atom {
            start: cursor,
            end: atom_end,
            kind: bytes[cursor + 4..cursor + 8].try_into()?,
        });
        cursor = atom_end;
    }
    ensure!(cursor == end, "MP4 container has trailing bytes");
    Ok(parsed)
}

fn patch_children(
    bytes: &mut [u8],
    start: usize,
    media: Range<usize>,
    delta: u64,
    control: &TransformControl,
) -> Result<usize> {
    let children = atoms(bytes, start, bytes.len(), control)?;
    let mut patched = 0;
    for child in children {
        patched += match child.kind {
            kind if kind == *b"stco" => {
                patch_offsets(bytes, child, media.clone(), delta, 4, control)?
            }
            kind if kind == *b"co64" => {
                patch_offsets(bytes, child, media.clone(), delta, 8, control)?
            }
            kind if is_container(kind) => {
                patch_container(bytes, child, media.clone(), delta, control)?
            }
            _ => 0,
        };
    }
    Ok(patched)
}

fn patch_container(
    bytes: &mut [u8],
    atom: Atom,
    media: Range<usize>,
    delta: u64,
    control: &TransformControl,
) -> Result<usize> {
    let end = atom.end;
    let children = atoms(bytes, atom.start + 8, end, control)?;
    let mut patched = 0;
    for child in children {
        patched += match child.kind {
            kind if kind == *b"stco" => {
                patch_offsets(bytes, child, media.clone(), delta, 4, control)?
            }
            kind if kind == *b"co64" => {
                patch_offsets(bytes, child, media.clone(), delta, 8, control)?
            }
            kind if is_container(kind) => {
                patch_container(bytes, child, media.clone(), delta, control)?
            }
            _ => 0,
        };
    }
    Ok(patched)
}

fn patch_offsets(
    bytes: &mut [u8],
    atom: Atom,
    media: Range<usize>,
    delta: u64,
    width: usize,
    control: &TransformControl,
) -> Result<usize> {
    let payload = atom.start + 8;
    ensure!(
        atom.end.saturating_sub(payload) >= 8,
        "truncated chunk-offset atom"
    );
    let count = read_u32(bytes, payload + 4)? as usize;
    let values = payload + 8;
    let required = count
        .checked_mul(width)
        .and_then(|size| values.checked_add(size))
        .context("chunk-offset table overflow")?;
    ensure!(required == atom.end, "malformed chunk-offset table");
    for index in 0..count {
        control.checkpoint()?;
        patch_offset(bytes, values + index * width, width, &media, delta)?;
    }
    Ok(count)
}

fn patch_offset(
    bytes: &mut [u8],
    at: usize,
    width: usize,
    media: &Range<usize>,
    delta: u64,
) -> Result<()> {
    let old = match width {
        4 => read_u32(bytes, at)? as u64,
        8 => u64::from_be_bytes(bytes[at..at + 8].try_into()?),
        _ => unreachable!("validated chunk offset width"),
    };
    ensure!(
        (media.start as u64..media.end as u64).contains(&old),
        "chunk offset is outside the sole mdat payload"
    );
    let new = old.checked_add(delta).context("chunk offset overflow")?;
    if width == 4 {
        bytes[at..at + 4].copy_from_slice(&u32::try_from(new)?.to_be_bytes());
    } else {
        bytes[at..at + 8].copy_from_slice(&new.to_be_bytes());
    }
    Ok(())
}

fn read_u32(bytes: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_be_bytes(
        bytes
            .get(at..at + 4)
            .context("truncated MP4 integer")?
            .try_into()?,
    ))
}

fn is_container(kind: [u8; 4]) -> bool {
    matches!(
        &kind,
        b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl" | b"dinf" | b"edts" | b"udta" | b"mvex"
    )
}
