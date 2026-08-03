import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('leased files remain cached until every surface releases them',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader({
        first.remoteUrl!: <int>[1, 2, 3, 4],
        second.remoteUrl!: <int>[5, 6, 7, 8],
      }),
      maxBytes: 4,
    );

    final firstLease = (await store.acquire(first))!;
    final secondLease = (await store.acquire(first))!;

    expect(await store.acquire(second), isNull);
    expect(await File(firstLease.media.localPath!).readAsBytes(), [1, 2, 3, 4]);
    firstLease.release();
    expect(await store.acquire(second), isNull);
    expect(await File(secondLease.media.localPath!).exists(), isTrue);

    secondLease.release();
    final cachedSecond = (await store.acquire(second))!;
    expect(cachedSecond.media.isLocal, isTrue);
    expect(await File(firstLease.media.localPath!).exists(), isFalse);
    cachedSecond.release();
  });
}
