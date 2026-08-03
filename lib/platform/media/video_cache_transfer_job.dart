part of 'video_cache_transfer_pool.dart';

class _VideoCacheTransferJob {
  _VideoCacheTransferJob(
    this.directory,
    this.availableBytes,
    this.evictOldest,
    this.transfer,
  );

  final Directory directory;
  final VideoCacheAvailableBytes availableBytes;
  final VideoCacheEviction evictOldest;
  final VideoCacheTransfer transfer;
  final Completer<bool> result = Completer<bool>();
  int lastGrantedBytes = -1;
  int minimumBytes = 1;
}
