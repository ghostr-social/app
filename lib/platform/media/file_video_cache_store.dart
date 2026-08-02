import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_store.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/video_cache_download_queue.dart';
import 'package:ghostr/platform/media/video_cache_directory.dart';
import 'package:ghostr/platform/media/video_cache_files.dart';

class FileVideoCacheStore implements VideoCacheStore {
  FileVideoCacheStore({
    required CacheDirectoryProvider directoryProvider,
    required VideoFileDownloader downloader,
    required this.maxBytes,
    Clock clock = systemClock,
  })  : _directoryProvider = directoryProvider,
        _downloader = downloader,
        _clock = clock;

  final CacheDirectoryProvider _directoryProvider;
  final VideoFileDownloader _downloader;
  final Clock _clock;
  final int maxBytes;
  final Set<String> _activePartialPaths = <String>{};
  final VideoCacheDownloadQueue _downloadQueue = VideoCacheDownloadQueue();
  late final VideoCacheDirectory _cacheDirectory = VideoCacheDirectory(
    maxBytes,
    _activePartialPaths,
  );
  int _requestId = 0;

  Future<void> initialize() async {
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
  Future<VideoMediaSource?> find(VideoMediaSource media) async {
    try {
      final directory = await _directoryProvider();
      await _cacheDirectory.maintain(directory);
      final file = File(_completedPath(directory, media));
      if (!await file.exists() || await file.length() == 0) return null;
      await file.setLastModified(_clock());
      return VideoMediaSource.local(file.path);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _cacheFailure(error, stackTrace);
    }
  }

  @override
  Future<VideoMediaSource?> download(VideoMediaSource media) {
    return _downloadQueue.run(() => _guardedDownload(media));
  }

  Future<VideoMediaSource?> _guardedDownload(VideoMediaSource media) async {
    try {
      return await _download(media);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw _cacheFailure(error, stackTrace);
    }
  }

  Future<VideoMediaSource?> _download(VideoMediaSource media) async {
    if (maxBytes <= 0) return null;
    final request = await _createRequest(media);
    _activePartialPaths.add(request.partial.path);
    try {
      return await _completeDownload(request);
    } on AppFailure {
      await _deleteIfPresent(request.partial);
      rethrow;
    } on Object catch (error, stackTrace) {
      await _deleteIfPresent(request.partial);
      throw _cacheFailure(error, stackTrace);
    } finally {
      _activePartialPaths.remove(request.partial.path);
    }
  }

  Future<VideoMediaSource?> _completeDownload(VideoCacheRequest request) async {
    if (!await _downloadWithinBudget(request)) return null;
    await _replaceCompletedFile(request);
    await _cacheDirectory.enforceBudget(request.directory);
    return _completedMedia(request.completed);
  }

  Future<VideoMediaSource?> _completedMedia(File completed) async {
    if (!await completed.exists()) return null;
    return VideoMediaSource.local(completed.path);
  }

  Future<bool> _downloadWithinBudget(VideoCacheRequest request) async {
    while (true) {
      final available = await _cacheDirectory.availableBytes(request.directory);
      try {
        await _downloadFromSources(request, available);
        return true;
      } on VideoDownloadLimitExceeded {
        if (!await _cacheDirectory.evictOldest(request.directory)) return false;
      }
    }
  }

  Future<void> _downloadFromSources(
    VideoCacheRequest request,
    int availableBytes,
  ) async {
    Object? lastError;
    for (final source in request.sources) {
      try {
        await _deleteIfPresent(request.partial);
        await _downloader.download(
          source,
          request.partial.path,
          maxBytes: availableBytes,
        );
        await _validateDownload(request.partial);
        return;
      } on Object catch (error, stackTrace) {
        logBoundaryFailure(
          source: 'ghostr.media.file-cache',
          message: 'A video cache source failed; trying its fallback.',
          error: error,
          stackTrace: stackTrace,
        );
        lastError = error;
      }
    }
    if (lastError is AppFailure) throw lastError;
    throw const AppFailure('The video could not be cached.');
  }

  Future<VideoCacheRequest> _createRequest(VideoMediaSource media) async {
    final source = Uri.tryParse(media.remoteUrl ?? '');
    final sources = media.remoteUrls.map(Uri.tryParse).whereType<Uri>().where(
          (uri) => uri.hasAuthority,
        );
    if (source == null || !source.hasAuthority || sources.isEmpty) {
      throw const AppFailure('The video URL cannot be cached.');
    }
    final directory = await _directoryProvider();
    await directory.create(recursive: true);
    await _cacheDirectory.maintain(directory);
    final completed = File(_completedPath(directory, media));
    return VideoCacheRequest(
      directory: directory,
      sources: sources.toList(),
      completed: completed,
      partial: File('${completed.path}.${_requestId++}.partial'),
    );
  }

  Future<void> _validateDownload(File file) async {
    if (!await file.exists() || await file.length() == 0) {
      throw const AppFailure('The downloaded video was empty.');
    }
  }

  Future<void> _replaceCompletedFile(VideoCacheRequest request) async {
    await _deleteIfPresent(request.completed);
    await request.partial.rename(request.completed.path);
  }

  String _completedPath(Directory directory, VideoMediaSource media) {
    final digest =
        sha256.convert(utf8.encode(media.remoteUrl ?? '')).toString();
    return '${directory.path}${Platform.pathSeparator}$digest.video';
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
