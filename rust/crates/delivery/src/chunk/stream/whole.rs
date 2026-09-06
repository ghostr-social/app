use super::{ended_whole, next_input, stopped, store_bytes, StoreInput, StreamInput, Streamed};
use crate::chunk::sink::ChunkWrite;
use ghostr_engine::adaptive::{WholeBodyContract, REQUEST_SLICE_BYTES};

#[derive(Default)]
struct Window { written: u64, through: u64 }

pub(super) async fn stream_whole<W: ChunkWrite + ?Sized>(
    input: &mut StreamInput<'_, '_, W>, contract: WholeBodyContract,
) -> anyhow::Result<Streamed> {
    let mut window = Window::default();
    loop {
        if !window.renew(input, contract).await? { return Ok(stopped(window.written)); }
        let Some(mut chunk) = next_input(input).await? else {
            let streamed = ended_whole(window.written, contract, input)?;
            if let Some(completion) = streamed.whole_body_completion.clone() {
                input.traffic.whole_body_completed(completion);
            }
            return Ok(streamed);
        };
        crate::chunk::whole_body_limit::WholeBodyLimitReached::check(window.written, chunk.len() as u64, contract)?;
        if !window.write(input, contract, &mut chunk).await? { return Ok(stopped(window.written)); }
    }
}

impl Window {
    async fn renew<W: ChunkWrite + ?Sized>(
        &mut self, input: &mut StreamInput<'_, '_, W>, contract: WholeBodyContract,
    ) -> anyhow::Result<bool> {
        if self.written < self.through || self.written == contract.maximum_bytes() { return Ok(true); }
        let through = self.written.saturating_add(REQUEST_SLICE_BYTES).min(contract.maximum_bytes());
        tokio::select! {
            biased;
            () = input.cancel.cancelled() => return Ok(false),
            granted = input.traffic.authorize_body(through) => granted?,
        }
        self.through = through;
        Ok(true)
    }

    async fn write<W: ChunkWrite + ?Sized>(
        &mut self, input: &mut StreamInput<'_, '_, W>, contract: WholeBodyContract, chunk: &mut bytes::Bytes,
    ) -> anyhow::Result<bool> {
        while !chunk.is_empty() {
            if !self.renew(input, contract).await? { return Ok(false); }
            let take = chunk.len().min((self.through - self.written) as usize);
            let part = chunk.split_to(take);
            let stored = store_bytes(StoreInput::from(&*input), self.written, &part).await?;
            self.written += stored.bytes;
            if stored.cancelled { return Ok(false); }
        }
        Ok(true)
    }
}
