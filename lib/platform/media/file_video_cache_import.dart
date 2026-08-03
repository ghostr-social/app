part of 'file_video_cache_store.dart';

extension _FileVideoCacheImport on FileVideoCacheStore {
  Future<bool> _importWithinBudget(
    VideoCacheRequest request,
    VideoMediaSource media,
    int maxBytes,
  ) async {
    final sourcePath = media.importPath;
    if (sourcePath == null) return false;
    return _sourceImporter.import(
      request,
      media,
      sourcePath,
      maxBytes,
    );
  }
}
