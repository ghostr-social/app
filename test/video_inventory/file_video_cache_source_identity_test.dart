import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('isolates same-primary media with different fallback identities',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const primary = 'https://media.test/video.mp4';
    const firstFallback = 'https://one.test/video.mp4';
    const secondFallback = 'https://two.test/video.mp4';
    final downloader = FakeVideoFileDownloader(
      const {
        firstFallback: [1],
        secondFallback: [2],
      },
      failingUrls: const {primary},
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = VideoMediaSource.remote(
      primary,
      fallbackUrls: const [firstFallback],
    );
    final second = VideoMediaSource.remote(
      primary,
      fallbackUrls: const [secondFallback],
    );

    final firstCached = (await store.acquire(first))!;
    final firstPath = firstCached.media.localPath;
    firstCached.release();

    final secondCached = (await store.acquire(second))!;
    expect(secondCached.media.localPath, isNot(firstPath));
    expect(await File(secondCached.media.localPath!).readAsBytes(), [2]);
    secondCached.release();
  });
}
