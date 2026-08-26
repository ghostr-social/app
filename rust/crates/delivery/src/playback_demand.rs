use core::num::NonZeroU64;
use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, PostId};
use tokio::sync::mpsc;

static NEXT_CONSUMER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConsumerId(NonZeroU64);

impl ConsumerId {
    pub fn new(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    fn next() -> Self {
        let value = NEXT_CONSUMER_ID.fetch_add(1, Ordering::Relaxed);
        Self::new(value).expect("playback consumer ID exhausted")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DemandLease {
    consumer: ConsumerId,
    post: PostId,
    representation: Option<RepresentationBinding>,
    range: ByteRange,
}

impl DemandLease {
    pub fn new(
        consumer: ConsumerId,
        post: PostId,
        representation: Option<RepresentationBinding>,
        range: ByteRange,
    ) -> Self {
        Self {
            consumer,
            post,
            representation,
            range,
        }
    }

    pub fn consumer(&self) -> ConsumerId {
        self.consumer
    }

    pub fn post(&self) -> &PostId {
        &self.post
    }

    pub(super) fn representation(&self) -> Option<&RepresentationBinding> {
        self.representation.as_ref()
    }

    pub fn range(&self) -> ByteRange {
        self.range
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DemandState {
    Blocked(DemandLease),
    Advanced(DemandLease),
    Released(ConsumerId),
}

#[derive(Clone, Debug)]
pub struct DemandSender(mpsc::UnboundedSender<DemandState>);

impl DemandSender {
    pub fn emit(&self, state: DemandState) {
        let _ = self.0.send(state);
    }

    pub fn consumer(
        &self,
        post: PostId,
        representation: Option<RepresentationBinding>,
    ) -> DemandConsumer {
        DemandConsumer {
            sender: self.clone(),
            id: ConsumerId::next(),
            post,
            representation,
            last_range: None,
        }
    }
}

pub struct DemandConsumer {
    sender: DemandSender,
    id: ConsumerId,
    post: PostId,
    representation: Option<RepresentationBinding>,
    last_range: Option<ByteRange>,
}

impl DemandConsumer {
    pub fn demand(&mut self, range: ByteRange) {
        if self.last_range == Some(range) {
            return;
        }
        let lease = DemandLease::new(
            self.id,
            self.post.clone(),
            self.representation.clone(),
            range,
        );
        let state = match self.last_range {
            Some(_) => DemandState::Advanced(lease),
            None => DemandState::Blocked(lease),
        };
        self.last_range = Some(range);
        self.sender.emit(state);
    }
}

impl Drop for DemandConsumer {
    fn drop(&mut self) {
        if self.last_range.is_some() {
            self.sender.emit(DemandState::Released(self.id));
        }
    }
}

pub type DemandReceiver = mpsc::UnboundedReceiver<DemandState>;

pub fn demand_channel() -> (DemandSender, DemandReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DemandSender(sender), receiver)
}
