part of 'file_video_cache_store.dart';

extension _FileVideoCacheTransfer on FileVideoCacheStore {
  Future<VideoCacheTransferResult> _transfer(
    VideoCacheRequest request,
    VideoMediaSource media,
    VideoCacheDeadline deadline,
    int maxTransferBytes,
  ) async {
    try {
      await _performTransfer(request, media, deadline, maxTransferBytes);
      return VideoCacheTransferResult.completed;
    } on VideoDownloadLimitExceeded {
      await _deleteIfPresent(request.partial);
      return VideoCacheTransferResult.retryWithMoreCapacity;
    } on Object {
      await _deleteIfPresent(request.partial);
      rethrow;
    }
  }

  Future<void> _performTransfer(
    VideoCacheRequest request,
    VideoMediaSource media,
    VideoCacheDeadline deadline,
    int maxTransferBytes,
  ) async {
    if (!await _importWithinBudget(request, media, maxTransferBytes)) {
      await _sourceDownloader.download(
        request,
        maxTransferBytes,
        media,
        deadline,
      );
    }
    await _requireWithinReservation(request.partial, maxTransferBytes);
    await _metadataQueue.run(() => _installCompleted(request));
  }

  Future<void> _requireWithinReservation(File partial, int maxBytes) async {
    if (await partial.length() > maxBytes) {
      throw const VideoDownloadLimitExceeded();
    }
  }

  Future<void> _installCompleted(VideoCacheRequest request) async {
    await _replaceCompletedFile(request);
    // The transfer already validated the digest, so the first playback
    // acquire must not hash the file again.
    _verifiedPaths.add(request.completed.path);
    await _cacheDirectory.enforceBudget(request.directory);
    _pendingLeasePaths.add(request.completed.path);
  }

  Future<int> _availableBytes(Directory directory) {
    return _metadataQueue.run(() => _cacheDirectory.availableBytes(directory));
  }

  Future<bool> _evictOldest(Directory directory) {
    return _metadataQueue.run(() => _cacheDirectory.evictOldest(directory));
  }

  Future<void> _releasePendingLease(String? path) {
    return _metadataQueue.run(() async {
      if (path != null) _pendingLeasePaths.remove(path);
    });
  }
}
