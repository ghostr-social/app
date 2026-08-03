import 'dart:io';

import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';

class FakeVideoFileDownloader implements VideoFileDownloader {
  FakeVideoFileDownloader(
    this.bytesByUrl, {
    this.error,
    this.failingUrls = const {},
  });

  final Map<String, List<int>> bytesByUrl;
  final Object? error;
  final Set<String> failingUrls;
  final List<String> attemptedUrls = [];

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    attemptedUrls.add(source.toString());
    if (error != null) throw error!;
    if (failingUrls.contains(source.toString())) throw StateError('offline');
    final bytes = bytesByUrl[source.toString()]!;
    if (bytes.length > maxBytes) {
      throw const VideoDownloadLimitExceeded();
    }
    await File(destinationPath).writeAsBytes(bytes);
  }
}
