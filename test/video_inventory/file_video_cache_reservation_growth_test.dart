import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('uses remaining capacity before evicting after a fair-share retry',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader({
        first.remoteUrl!: [1, 2, 3, 4, 5],
        second.remoteUrl!: [6, 7, 8, 9],
      }),
      maxBytes: 10,
    );

    final firstResult = store.acquire(first);
    final secondResult = store.acquire(second);
    final firstLease = await firstResult;
    final secondLease = await secondResult;

    expect(firstLease, isNotNull);
    expect(secondLease, isNotNull);
    firstLease?.release();
    secondLease?.release();
  });
}
