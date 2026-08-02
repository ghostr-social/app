import 'dart:io';

import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:http/http.dart' as http;

class HttpVideoFileDownloader implements VideoFileDownloader {
  const HttpVideoFileDownloader(this._client);

  final http.Client _client;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
  }) async {
    try {
      final response = await _client.send(http.Request('GET', source));
      await _requireSuccess(response);
      await _writeWithinLimit(response.stream, destinationPath, maxBytes);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.media.http-cache',
        message: 'The video could not be cached.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<void> _requireSuccess(http.StreamedResponse response) async {
    if (response.statusCode >= 200 && response.statusCode < 300) return;
    await response.stream.drain<void>();
    throw AppFailure('Video download failed (${response.statusCode}).');
  }

  Future<void> _writeWithinLimit(
    Stream<List<int>> stream,
    String destinationPath,
    int maxBytes,
  ) async {
    final sink = File(destinationPath).openWrite();
    var written = 0;
    try {
      await for (final chunk in stream) {
        if (written + chunk.length > maxBytes) {
          throw const VideoDownloadLimitExceeded();
        }
        sink.add(chunk);
        written += chunk.length;
      }
    } finally {
      await sink.close();
    }
  }
}
