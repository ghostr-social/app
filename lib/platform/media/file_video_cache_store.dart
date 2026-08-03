import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_cache_identity.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_lease.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/video_cache_directory.dart';
import 'package:ghostr/platform/media/video_cache_files.dart';
import 'package:ghostr/platform/media/video_cache_flight_registry.dart';
import 'package:ghostr/platform/media/video_cache_integrity.dart';
import 'package:ghostr/platform/media/video_cache_lease_registry.dart';
import 'package:ghostr/platform/media/video_cache_media_files.dart';
import 'package:ghostr/platform/media/video_cache_metadata_lock.dart';
import 'package:ghostr/platform/media/video_cache_source_downloader.dart';
import 'package:ghostr/platform/media/video_cache_source_importer.dart';
import 'package:ghostr/platform/media/video_cache_store_timing.dart';
import 'package:ghostr/platform/media/video_cache_transfer_pool.dart';

part 'file_video_cache_import.dart';
part 'file_video_cache_transfer.dart';

class FileVideoCacheStore implements VideoCacheStore {
  FileVideoCacheStore({
    required CacheDirectoryProvider directoryProvider,
    required VideoFileDownloader downloader,
    required this.maxBytes,
    VideoCacheStoreTiming timing = const VideoCacheStoreTiming(),
    int maxConcurrentTransfers =
        VideoCacheTransferPool.defaultMaxConcurrentTransfers,
  })  : _directoryProvider = directoryProvider,
        _sourceDownloader = VideoCacheSourceDownloader(downloader),
        _timing = timing,
        _transferPool = VideoCacheTransferPool(
          maxBytes: maxBytes,
          maxConcurrentTransfers: maxConcurrentTransfers,
        );

  final CacheDirectoryProvider _directoryProvider;
  final VideoCacheSourceDownloader _sourceDownloader;
  final VideoCacheSourceImporter _sourceImporter =
      const VideoCacheSourceImporter();
  final VideoCacheStoreTiming _timing;
  final VideoCacheTransferPool _transferPool;
  final int maxBytes;
  final Set<String> _activePartialPaths = <String>{};
  final Set<String> _pendingLeasePaths = <String>{};
  final VideoCacheLeaseRegistry _leases = VideoCacheLeaseRegistry();
  final VideoCacheFlightRegistry<VideoMediaSource?> _flights =
      VideoCacheFlightRegistry<VideoMediaSource?>();
  final VideoCacheMetadataLock _metadataQueue = VideoCacheMetadataLock();
  late final VideoCacheDirectory _cacheDirectory = VideoCacheDirectory(
    maxBytes,
    _activePartialPaths,
    _leases.activePaths,
    pendingLeasePaths: _pendingLeasePaths,
  );
  int _requestId = 0;

  Future<void> initialize() {
    return _metadataQueue.run(_initialize);
  }

  Future<void> _initialize() async {
    try {
      final directory = await _directoryProvider();
      await _cacheDirectory.maintain(directory);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _cacheFailure(error, stackTrace);
    }
  }

  @override
  Future<VideoCacheLease?> acquire(VideoMediaSource media) async {
    if (maxBytes <= 0) return null;
    final existing = await _acquireExisting(media);
    if (existing != null) return existing;
    return _acquireDownloaded(media);
  }

  Future<VideoCacheLease?> _acquireDownloaded(VideoMediaSource media) async {
    final key = media.cacheStorageIdentity;
    final registration = _flights.join(
      key,
      media.cacheJobIdentity,
      () => _download(media),
    );
    VideoMediaSource? cached;
    try {
      cached = await registration.flight.result;
      if (cached == null) return null;
      return await _acquireExisting(media);
    } on Object {
      if (!registration.retryOnFailure) rethrow;
    } finally {
      if (_flights.leave(registration.flight)) {
        await _releasePendingLease(cached?.localPath);
      }
    }
    return acquire(media);
  }

  Future<VideoCacheLease?> _acquireExisting(VideoMediaSource media) {
    return _metadataQueue.run(() async {
      final cached = await _find(media);
      return cached == null ? null : _leases.acquire(cached);
    });
  }

  Future<VideoMediaSource?> _find(VideoMediaSource media) async {
    try {
      final directory = await _directoryProvider();
      await _cacheDirectory.maintain(directory);
      final file = File(completedVideoCachePath(directory, media));
      if (!await validateExistingVideoCache(file, media.expectedSha256)) {
        return null;
      }
      await file.setLastModified(_timing.accessClock());
      return completedVideoCacheMedia(file, media);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _cacheFailure(error, stackTrace);
    }
  }

  Future<VideoMediaSource?> _download(VideoMediaSource media) async {
    final deadline = _timing.startSourceSet();
    final request = await _createRequest(media);
    _activePartialPaths.add(request.partial.path);
    try {
      final completed = await _transferPool.run(
        directory: request.directory,
        availableBytes: _availableBytes,
        evictOldest: _evictOldest,
        transfer: (limit) => _transfer(request, media, deadline, limit),
      );
      return completed
          ? completedVideoCacheMedia(request.completed, media)
          : null;
    } on AppFailure {
      await _deleteIfPresent(request.partial);
      await _releasePendingLease(request.completed.path);
      rethrow;
    } on Object catch (error, stackTrace) {
      await _deleteIfPresent(request.partial);
      await _releasePendingLease(request.completed.path);
      throw _cacheFailure(error, stackTrace);
    } finally {
      _activePartialPaths.remove(request.partial.path);
    }
  }

  Future<VideoCacheRequest> _createRequest(VideoMediaSource media) async {
    final source = Uri.tryParse(media.remoteUrl ?? '');
    final sources =
        media.cacheSourceUrls.map(Uri.tryParse).whereType<Uri>().where(
              (uri) => uri.hasAuthority,
            );
    if (source == null || !source.hasAuthority || sources.isEmpty) {
      throw const AppFailure('The video URL cannot be cached.');
    }
    final directory = await _directoryProvider();
    await directory.create(recursive: true);
    await _metadataQueue.run(() => _cacheDirectory.maintain(directory));
    final completed = File(completedVideoCachePath(directory, media));
    return VideoCacheRequest(
      directory: directory,
      sources: sources.toList(),
      completed: completed,
      partial: File('${completed.path}.${_requestId++}.partial'),
    );
  }

  Future<void> _replaceCompletedFile(VideoCacheRequest request) async {
    await _deleteIfPresent(request.completed);
    await request.partial.rename(request.completed.path);
  }

  Future<void> _deleteIfPresent(File file) async {
    if (await file.exists()) await file.delete();
  }

  AppFailure _cacheFailure(Object error, StackTrace stackTrace) {
    return translatedBoundaryFailure(
      source: 'ghostr.media.file-cache',
      message: 'The video could not be cached.',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
