use core::ops::Range;

pub(super) fn whole(total: u64) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\n\
         Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\n\
         ETag: \"fixture-cancellable\"\r\n\r\n"
    )
}

pub(super) fn ranged(range: &Range<u64>, total: u64) -> String {
    format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{total}\r\n\
         Content-Length: {}\r\nContent-Type: video/mp4\r\nAccept-Ranges: bytes\r\n\
         ETag: \"fixture-cancellable\"\r\n\r\n",
        range.start,
        range.end - 1,
        range.end - range.start
    )
}
