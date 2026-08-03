import 'dart:io';

import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/platform/media/video_cache_files.dart';
import 'package:ghostr/platform/media/video_cache_integrity.dart';

class VideoCacheSourceImporter {
  const VideoCacheSourceImporter();

  Future<bool> import(
    VideoCacheRequest request,
    VideoMediaSource media,
    String sourcePath,
    int maxBytes,
  ) async {
    try {
      await _copyWithinLimit(
        File(sourcePath),
        request.partial,
        maxBytes,
      );
      await validateVideoCacheDownload(request.partial, media.expectedSha256);
      return true;
    } on VideoDownloadLimitExceeded {
      await _deletePartial(request.partial);
      rethrow;
    } on Object catch (error, stackTrace) {
      await _deletePartial(request.partial);
      _logUnavailable(error, stackTrace);
      return false;
    }
  }

  Future<void> _copyWithinLimit(
    File source,
    File destination,
    int maxBytes,
  ) async {
    final sink = destination.openWrite();
    var copiedBytes = 0;
    try {
      await for (final chunk in source.openRead()) {
        copiedBytes += chunk.length;
        if (copiedBytes > maxBytes) {
          throw const VideoDownloadLimitExceeded();
        }
        sink.add(chunk);
      }
    } finally {
      await sink.close();
    }
  }

  Future<void> _deletePartial(File partial) async {
    if (await partial.exists()) await partial.delete();
  }

  void _logUnavailable(Object error, StackTrace stackTrace) {
    logBoundaryFailure(
      source: 'ghostr.media.file-cache',
      message: 'Native cache import failed; trying the public source.',
      error: error,
      stackTrace: stackTrace,
    );
  }
}
