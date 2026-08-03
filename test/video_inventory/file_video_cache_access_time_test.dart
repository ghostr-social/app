import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/video_cache_store_timing.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('touches a cached video with the supplied access time', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');
    final downloader = FakeVideoFileDownloader({
      remote.debugLabel: [1, 2],
    });
    final cachedAt = DateTime.utc(2026, 8, 2, 12, 30);
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
      timing: VideoCacheStoreTiming(accessClock: () => cachedAt),
    );
    final cached = (await store.acquire(remote))!;
    cached.release();

    final accessed = (await store.acquire(remote))!;
    accessed.release();

    final modified = await File(cached.media.localPath!).lastModified();
    expect(modified.toUtc(), cachedAt);
  });
}
