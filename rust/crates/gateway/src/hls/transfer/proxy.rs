use super::HlsTransfer;
use crate::hls::asset_response::AssetBodyContract;
use axum::body::Body;
use bytes::Bytes;
use futures_util::Stream;
use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Instant};

struct AssetBodyState {
    transfer: HlsTransfer,
    contract: AssetBodyContract,
    sent: u64,
}

struct ProxyStream {
    receiver: mpsc::Receiver<Bytes>,
    terminal: Terminal,
    finished: bool,
}

type BodyResult = Result<Bytes, io::Error>;
type Terminal = Arc<Mutex<Option<io::Error>>>;

enum PumpStep {
    Chunk(Bytes),
    Finished,
    Failed(io::ErrorKind, String),
}

pub(super) fn body(transfer: HlsTransfer, contract: AssetBodyContract) -> Body {
    let state = AssetBodyState {
        transfer,
        contract,
        sent: 0,
    };
    let (sender, receiver) = mpsc::channel(1);
    let terminal = Arc::new(Mutex::new(None));
    tokio::spawn(pump(state, sender, Arc::clone(&terminal)));
    Body::from_stream(ProxyStream {
        receiver,
        terminal,
        finished: false,
    })
}

async fn pump(mut state: AssetBodyState, sender: mpsc::Sender<Bytes>, terminal: Terminal) {
    loop {
        match next_step(&mut state, &sender).await {
            PumpStep::Chunk(chunk) => {
                let deadline = state.transfer.total_deadline;
                if !send_chunk(&sender, chunk, deadline, &terminal).await {
                    return;
                }
            }
            PumpStep::Finished => return,
            PumpStep::Failed(kind, message) => return fail(&terminal, kind, &message),
        }
    }
}

async fn next_step(state: &mut AssetBodyState, sender: &mpsc::Sender<Bytes>) -> PumpStep {
    let next = tokio::select! {
        _ = sender.closed() => return PumpStep::Finished,
        next = state.transfer.next_chunk() => next,
    };
    classify(state, next)
}

fn classify(state: &mut AssetBodyState, next: anyhow::Result<Option<Bytes>>) -> PumpStep {
    match next {
        Ok(Some(chunk)) => classify_chunk(state, chunk),
        Ok(None) => classify_end(state),
        Err(error) => PumpStep::Failed(io::ErrorKind::TimedOut, error.to_string()),
    }
}

fn classify_chunk(state: &mut AssetBodyState, chunk: Bytes) -> PumpStep {
    match state.accepts(&chunk) {
        true => PumpStep::Chunk(chunk),
        false => PumpStep::Failed(
            io::ErrorKind::InvalidData,
            "HLS body exceeds its extent".to_owned(),
        ),
    }
}

fn classify_end(state: &AssetBodyState) -> PumpStep {
    match state.contract.complete(state.sent) {
        true => PumpStep::Finished,
        false => PumpStep::Failed(
            io::ErrorKind::UnexpectedEof,
            "HLS body ended early".to_owned(),
        ),
    }
}

async fn send_chunk(
    sender: &mpsc::Sender<Bytes>,
    chunk: Bytes,
    deadline: Instant,
    terminal: &Terminal,
) -> bool {
    tokio::select! {
        biased;
        _ = sleep_until(deadline) => {
            fail(terminal, io::ErrorKind::TimedOut, "HLS object transfer timed out");
            false
        }
        _ = sender.closed() => false,
        result = sender.send(chunk) => result.is_ok(),
    }
}

impl AssetBodyState {
    fn accepts(&mut self, chunk: &Bytes) -> bool {
        let Some(total) = self.contract.checked_total(self.sent, chunk.len()) else {
            return false;
        };
        self.sent = total;
        true
    }
}

impl Stream for ProxyStream {
    type Item = BodyResult;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(context) {
            Poll::Ready(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(None) => Poll::Ready(self.terminal()),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl ProxyStream {
    fn terminal(&mut self) -> Option<BodyResult> {
        if self.finished {
            return None;
        }
        self.finished = true;
        lock(&self.terminal).take().map(Err)
    }
}

fn fail(terminal: &Terminal, kind: io::ErrorKind, message: &str) {
    *lock(terminal) = Some(io::Error::new(kind, message.to_owned()));
}

fn lock(terminal: &Terminal) -> std::sync::MutexGuard<'_, Option<io::Error>> {
    terminal
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
