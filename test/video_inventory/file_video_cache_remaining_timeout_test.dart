import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/video_cache_store_timing.dart';

void main() {
  test('passes each fallback only the source-set time still remaining',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    var elapsed = Duration.zero;
    final downloader = _AdvancingDownloader(
      advance: () => elapsed = const Duration(seconds: 6),
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
      timing: VideoCacheStoreTiming(
        elapsedClock: () => elapsed,
        sourceSetTimeout: const Duration(seconds: 10),
      ),
    );
    final media = VideoMediaSource.remote(
      'https://media.test/video.mp4',
      fallbackUrls: const ['https://mirror.test/video.mp4'],
    );

    final cached = (await store.acquire(media))!;

    expect(cached.media.isLocal, isTrue);
    expect(downloader.timeouts, [
      const Duration(seconds: 10),
      const Duration(seconds: 4),
    ]);
    cached.release();
  });
}

class _AdvancingDownloader implements VideoFileDownloader {
  _AdvancingDownloader({required this.advance});

  final void Function() advance;
  final List<Duration?> timeouts = [];

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    timeouts.add(totalTimeout);
    if (source.host == 'media.test') {
      advance();
      throw const AppFailure('Primary offline.');
    }
    await File(destinationPath).writeAsBytes(const [1]);
  }
}
