use super::PreparedObject;
use crate::segmented::cache::{AssemblySeed, CachedHlsGenerationHasher};
use crate::segmented::CachedHlsGeneration;
use std::sync::Arc;
use tokio::sync::oneshot;

const ASSEMBLY_QUANTUM: usize = 512 * 1024;

struct AssemblyBytes {
    prefix: Vec<Arc<[u8]>>,
    block: Arc<[u8]>,
    length: usize,
}

struct AssemblyWriter<'a> {
    output: &'a mut Vec<u8>,
    limit: usize,
    hasher: &'a mut CachedHlsGenerationHasher,
}

pub(in crate::segmented) struct PreparedComplete {
    pub(in crate::segmented) object: PreparedObject,
    pub(in crate::segmented) generation: CachedHlsGeneration,
}

impl PreparedComplete {}

pub(in crate::segmented) async fn prepare_complete(
    seed: Option<AssemblySeed>,
    block: PreparedObject,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<PreparedComplete, ()> {
    match seed {
        Some(seed) => assemble(seed, block, cancelled).await,
        None => prehash(block, cancelled).await,
    }
}

async fn prehash(
    object: PreparedObject,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<PreparedComplete, ()> {
    let mut hasher =
        CachedHlsGenerationHasher::new(&object.final_url, object.body.len() as u64, &object.cache);
    check_cancelled(cancelled)?;
    for chunk in object.body.chunks(ASSEMBLY_QUANTUM) {
        hasher.update(chunk);
        checkpoint(cancelled).await?;
    }
    Ok(PreparedComplete {
        object,
        generation: hasher.finish(),
    })
}

async fn assemble(
    seed: AssemblySeed,
    block: PreparedObject,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<PreparedComplete, ()> {
    let total = seed.bytes.checked_add(block.body.len() as u64).ok_or(())?;
    let Ok(length) = usize::try_from(total) else {
        return Err(());
    };
    let cache = seed.cache.combined_with(&block.cache);
    let mut hasher = CachedHlsGenerationHasher::new(&seed.final_url, total, &cache);
    check_cancelled(cancelled)?;
    let bytes = AssemblyBytes {
        prefix: seed.blocks,
        block: block.body,
        length,
    };
    let body = assemble_bytes(bytes, &mut hasher, cancelled).await?;
    Ok(PreparedComplete {
        object: PreparedObject {
            request_url: seed.request_url,
            final_url: seed.final_url,
            body,
            content_type: seed.content_type,
            cache,
        },
        generation: hasher.finish(),
    })
}

async fn assemble_bytes(
    bytes: AssemblyBytes,
    hasher: &mut CachedHlsGenerationHasher,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<Arc<[u8]>, ()> {
    let mut body = Vec::with_capacity(bytes.length);
    copy_sources(&mut body, &bytes, hasher, cancelled).await?;
    if body.len() != bytes.length {
        return Err(());
    }
    Ok(body.into())
}

async fn copy_sources(
    body: &mut Vec<u8>,
    bytes: &AssemblyBytes,
    hasher: &mut CachedHlsGenerationHasher,
    cancelled: &mut oneshot::Receiver<()>,
) -> Result<(), ()> {
    let mut writer = AssemblyWriter {
        output: body,
        limit: bytes.length,
        hasher,
    };
    for source in bytes.prefix.iter().chain(core::iter::once(&bytes.block)) {
        writer.copy_source(source, cancelled).await?;
    }
    Ok(())
}

impl AssemblyWriter<'_> {
    async fn copy_source(
        &mut self,
        source: &[u8],
        cancelled: &mut oneshot::Receiver<()>,
    ) -> Result<(), ()> {
        for chunk in source.chunks(ASSEMBLY_QUANTUM) {
            write_chunk(self.output, self.limit, chunk)?;
            self.hasher.update(chunk);
            checkpoint(cancelled).await?;
        }
        Ok(())
    }
}

fn write_chunk(output: &mut Vec<u8>, limit: usize, chunk: &[u8]) -> Result<(), ()> {
    let end = output.len().checked_add(chunk.len()).ok_or(())?;
    (end <= limit).then_some(()).ok_or(())?;
    output.extend_from_slice(chunk);
    Ok(())
}

async fn checkpoint(cancelled: &mut oneshot::Receiver<()>) -> Result<(), ()> {
    tokio::select! {
        biased;
        _ = &mut *cancelled => Err(()),
        () = tokio::task::yield_now() => Ok(()),
    }
}

fn check_cancelled(cancelled: &mut oneshot::Receiver<()>) -> Result<(), ()> {
    match cancelled.try_recv() {
        Ok(()) | Err(oneshot::error::TryRecvError::Closed) => Err(()),
        Err(oneshot::error::TryRecvError::Empty) => Ok(()),
    }
}

#[cfg(test)]
#[path = "completion_axiom_test.rs"]
pub(crate) mod axiom_test_support;
