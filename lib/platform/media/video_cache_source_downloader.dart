import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/video_cache_files.dart';
import 'package:ghostr/platform/media/video_cache_integrity.dart';
import 'package:ghostr/platform/media/video_cache_store_timing.dart';

class VideoCacheSourceDownloader {
  const VideoCacheSourceDownloader(this._downloader);

  final VideoFileDownloader _downloader;

  Future<void> download(
    VideoCacheRequest request,
    int availableBytes,
    VideoMediaSource media,
    VideoCacheDeadline deadline,
  ) async {
    final failures = _VideoCacheSourceFailures();
    for (final source in request.sources) {
      final failure = await _captureFailure(source, () async {
        await _deleteIfPresent(request.partial);
        await _downloadCandidate(
          source,
          request.partial,
          availableBytes,
          deadline,
        );
        await validateVideoCacheDownload(
          request.partial,
          media.expectedSha256,
        );
        deadline.requireActive();
      });
      if (failure == null) return;
      failures.record(failure);
    }
    deadline.requireActive();
    failures.throwLast();
  }

  Future<Object?> _captureFailure(
    Uri source,
    Future<void> Function() operation,
  ) async {
    try {
      await operation();
      return null;
    } on VideoCacheSourceSetTimedOut {
      rethrow;
    } on Object catch (error, stackTrace) {
      _logFailure(source, error, stackTrace);
      return error;
    }
  }

  Future<void> _downloadCandidate(
    Uri source,
    File partial,
    int availableBytes,
    VideoCacheDeadline deadline,
  ) async {
    await _downloader.download(
      source,
      partial.path,
      maxBytes: availableBytes,
      totalTimeout: deadline.remaining,
    );
  }

  void _logFailure(Uri source, Object error, StackTrace stackTrace) {
    logBoundaryFailure(
      source: 'ghostr.media.file-cache',
      message: 'A video cache source failed; trying its fallback.',
      error: error,
      stackTrace: stackTrace,
    );
  }

  Future<void> _deleteIfPresent(File file) async {
    if (await file.exists()) await file.delete();
  }
}

class _VideoCacheSourceFailures {
  Object? _last;
  VideoDownloadLimitExceeded? _limit;

  void record(Object failure) {
    if (failure is VideoDownloadLimitExceeded) _limit = failure;
    _last = failure;
  }

  Never throwLast() {
    final limit = _limit;
    if (limit != null) throw limit;
    final last = _last;
    if (last is AppFailure) throw last;
    throw const AppFailure('The video could not be cached.');
  }
}
