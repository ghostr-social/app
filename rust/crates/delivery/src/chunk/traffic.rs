use std::time::Duration;

pub trait ChunkTraffic: Send {
    fn opened(&mut self, ttfb: Duration);
    fn wrote(&mut self, bytes: u64);
}
