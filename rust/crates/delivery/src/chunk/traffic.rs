use std::time::Duration;

pub(crate) trait ChunkTraffic: Send {
    fn opened(&mut self, ttfb: Duration);
    fn wrote(&mut self, bytes: u64);
}

pub(crate) struct NoopTraffic;

impl ChunkTraffic for NoopTraffic {
    fn opened(&mut self, _ttfb: Duration) {}

    fn wrote(&mut self, _bytes: u64) {}
}
