struct Layout {
    sample_count: usize,
    sample_entries: usize,
    mdat_start: usize,
    mdat_size: usize,
    payload_start: usize,
    payload_end: usize,
}

pub(super) fn expand_avc_samples(input: &[u8], extra_bytes: usize) -> Vec<u8> {
    let layout = read_layout(input);
    let extra_total = layout.sample_count * extra_bytes;
    let mut output = vec![0; input.len() + extra_total];
    output[..layout.payload_start].copy_from_slice(&input[..layout.payload_start]);
    let target = copy_samples(input, &mut output, &layout, extra_bytes);
    output[target..].copy_from_slice(&input[layout.payload_end..]);
    write_u32(
        &mut output,
        layout.mdat_start,
        layout.mdat_size + extra_total,
    );
    output
}

fn read_layout(bytes: &[u8]) -> Layout {
    let stsz_type = find(bytes, b"stsz");
    let mdat_type = find(bytes, b"mdat");
    let mdat_start = mdat_type - 4;
    let mdat_size = read_u32(bytes, mdat_start);
    Layout {
        sample_count: read_u32(bytes, stsz_type + 12),
        sample_entries: stsz_type + 16,
        mdat_start,
        mdat_size,
        payload_start: mdat_type + 4,
        payload_end: mdat_start + mdat_size,
    }
}

fn copy_samples(input: &[u8], output: &mut [u8], layout: &Layout, extra: usize) -> usize {
    let mut source = layout.payload_start;
    let mut target = layout.payload_start;
    for index in 0..layout.sample_count {
        let entry = layout.sample_entries + index * 4;
        let size = read_u32(input, entry);
        output[target..target + size].copy_from_slice(&input[source..source + size]);
        append_filler(output, target + size, extra);
        write_u32(output, entry, size + extra);
        source += size;
        target += size + extra;
    }
    target
}

fn append_filler(output: &mut [u8], offset: usize, byte_length: usize) {
    let nal_size = byte_length - 4;
    write_u32(output, offset, nal_size);
    output[offset + 4] = 0x0c;
    output[offset + 5..offset + 3 + nal_size].fill(0xff);
    output[offset + 3 + nal_size] = 0x80;
}

fn find(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("MP4 fixture box")
}

fn read_u32(bytes: &[u8], offset: usize) -> usize {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().expect("u32 bytes")) as usize
}

fn write_u32(bytes: &mut [u8], offset: usize, value: usize) {
    bytes[offset..offset + 4].copy_from_slice(&(value as u32).to_be_bytes());
}
