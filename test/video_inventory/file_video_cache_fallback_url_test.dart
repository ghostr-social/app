import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('downloads a NIP-71 fallback after the primary URL fails', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const primary = 'https://media.test/primary.mp4';
    const fallback = 'https://media.test/fallback.mp4';
    final downloader = FakeVideoFileDownloader(
      const {
        fallback: [1, 2, 3]
      },
      failingUrls: const {primary},
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final media = VideoMediaSource.remote(
      primary,
      fallbackUrls: [fallback],
    );

    final cached = await store.download(media);

    expect(cached?.isLocal, isTrue);
    expect(downloader.attemptedUrls, [primary, fallback]);
  });
}
