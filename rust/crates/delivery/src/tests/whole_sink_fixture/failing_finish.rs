pub(crate) struct FailingFinishSink;

impl crate::chunk::downloader::ChunkWrite for FailingFinishSink {
    fn accept<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
    ) -> impl core::future::Future<Output = anyhow::Result<()>> + Send + 'a {
        core::future::ready(Ok(()))
    }

    fn write<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
        _offset: u64,
        _bytes: &'a [u8],
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send + 'a {
        core::future::ready(Ok(true))
    }

    fn finish<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
        _total: Option<u64>,
        _complete: bool,
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send + 'a {
        core::future::ready(Err(anyhow::anyhow!("forced rollback failure")))
    }
}
