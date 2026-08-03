import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/video_cache_store_timing.dart';

void main() {
  test('shares one deadline across every cache source candidate', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    var elapsed = Duration.zero;
    final downloader = _ExpiringDownloader(
      onFirstAttempt: () => elapsed = const Duration(seconds: 11),
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

    await expectLater(store.acquire(media), throwsA(isA<AppFailure>()));

    expect(downloader.attempts, 1);
    expect(downloader.timeouts, [const Duration(seconds: 10)]);
  });
}

class _ExpiringDownloader implements VideoFileDownloader {
  _ExpiringDownloader({required this.onFirstAttempt});

  final void Function() onFirstAttempt;
  final List<Duration?> timeouts = [];
  int attempts = 0;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    attempts += 1;
    timeouts.add(totalTimeout);
    onFirstAttempt();
    throw const AppFailure('Source offline.');
  }
}
