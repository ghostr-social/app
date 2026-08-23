pub(crate) struct FailingFinishSink;

impl crate::chunk::downloader::ChunkWrite for FailingFinishSink {
    async fn accept<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn write<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
        _offset: u64,
        _bytes: &'a [u8],
    ) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn finish<'a>(
        &'a self,
        _generation: &'a crate::chunk::generation::OriginGeneration,
        _mode: crate::chunk::downloader::ResponseWriteMode,
        _total: Option<u64>,
        _complete: bool,
    ) -> anyhow::Result<bool> {
        anyhow::bail!("forced rollback failure")
    }
}
