import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_download_limit_exceeded.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('preserves a candidate limit failure so eviction can retry', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    await File('${directory.path}/old.video').writeAsBytes(const [9]);
    final downloader = _LimitThenSuccessDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 1,
    );
    final media = VideoMediaSource.remote(
      'https://media.test/video.mp4',
      fallbackUrls: const ['https://mirror.test/video.mp4'],
    );

    final cached = (await store.acquire(media))!;

    expect(cached.media.isLocal, isTrue);
    expect(downloader.primaryAttempts, 2);
    cached.release();
  });
}

class _LimitThenSuccessDownloader implements VideoFileDownloader {
  int primaryAttempts = 0;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    if (source.host == 'mirror.test') {
      throw const AppFailure('Mirror offline.');
    }
    primaryAttempts += 1;
    if (primaryAttempts == 1) throw const VideoDownloadLimitExceeded();
    await File(destinationPath).writeAsBytes(const [1]);
  }
}
