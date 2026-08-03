import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('a corrupt source set cannot block a later shared-digest source set',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const primary = 'https://media.test/video.mp4';
    const corrupt = 'https://corrupt.test/video.mp4';
    const genuine = 'https://genuine.test/video.mp4';
    final downloader = FakeVideoFileDownloader(
      const {
        corrupt: [1],
        genuine: [2],
      },
      failingUrls: const {primary},
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = _media(primary, corrupt);
    final second = _media(primary, genuine);

    await expectLater(store.acquire(first), throwsA(isA<AppFailure>()));
    final cached = (await store.acquire(second))!;

    expect(await File(cached.media.localPath!).readAsBytes(), [2]);
    expect(downloader.attemptedUrls, [primary, corrupt, primary, genuine]);
    cached.release();
  });
}

VideoMediaSource _media(String primary, String fallback) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(primary, fallbackUrls: [fallback]),
    'dbc1b4c900ffe48d575b5da5c6380401'
    '25f65db0fe3e24494b76ea986457d986',
  );
}
