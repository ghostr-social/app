import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('restores verified cached media with its remote metadata', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const url = 'https://media.test/video.mp4';
    const mirror = 'https://mirror.test/video.mp4';
    final media = VideoMediaSource.withCacheScope(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.remote(url, fallbackUrls: const [mirror]),
        '4bf5122f344554c53bde2ebb8cd2b7e3d'
        '1600ad631c385a5d7cce23c7785459a',
      ),
      'event-revision-1',
    );
    final downloader = FakeVideoFileDownloader(const {
      url: [1]
    });
    final store = _store(directory, downloader);

    final cached = (await store.acquire(media))!;
    cached.release();
    final restored = (await _store(directory, downloader).acquire(media))!;

    expect(restored.media.isLocal, isTrue);
    expect(restored.media.remoteUrls, [url, mirror]);
    expect(restored.media.expectedSha256?.value, media.expectedSha256?.value);
    expect(restored.media.cacheScope?.value, media.cacheScope?.value);
    restored.release();
  });
}

FileVideoCacheStore _store(
  Directory directory,
  FakeVideoFileDownloader downloader,
) {
  return FileVideoCacheStore(
    directoryProvider: () async => directory,
    downloader: downloader,
    maxBytes: 10,
  );
}
