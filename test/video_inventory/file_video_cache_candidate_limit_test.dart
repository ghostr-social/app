import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('attempts at most the primary and four fallback cache sources',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final media = VideoMediaSource.remote(
      'https://media.test/video.mp4',
      fallbackUrls: List.generate(
        6,
        (index) => 'https://mirror$index.test/video.mp4',
      ),
    );
    final downloader = FakeVideoFileDownloader(
      const {},
      error: StateError('offline'),
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );

    await expectLater(store.acquire(media), throwsA(isA<AppFailure>()));

    expect(media.remoteUrls, hasLength(7));
    expect(downloader.attemptedUrls, media.remoteUrls.take(5));
  });
}
