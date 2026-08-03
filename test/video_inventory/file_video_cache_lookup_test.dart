import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('finds a completed cached video after the store is recreated', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');
    final downloader = FakeVideoFileDownloader({
      remote.debugLabel: [1, 2]
    });
    final firstStore = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final cached = (await firstStore.acquire(remote))!;
    final cachedPath = cached.media.localPath;
    cached.release();
    final recreatedStore = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );

    final restored = (await recreatedStore.acquire(remote))!;

    expect(restored.media.localPath, cachedPath);
    restored.release();
  });
}
