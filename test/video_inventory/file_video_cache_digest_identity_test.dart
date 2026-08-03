import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('isolates same-primary cache entries with different digests', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const primary = 'https://media.test/video.mp4';
    const mirror = 'https://mirror.test/video.mp4';
    final downloader = FakeVideoFileDownloader(const {
      primary: [1],
      mirror: [2],
    });
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = _media(primary, const [], _firstDigest);
    final second = _media(primary, const [mirror], _secondDigest);

    final firstCached = (await store.acquire(first))!;
    final firstPath = firstCached.media.localPath;
    firstCached.release();

    final secondCached = (await store.acquire(second))!;
    expect(secondCached.media.localPath, isNot(firstPath));
    expect(await File(secondCached.media.localPath!).readAsBytes(), [2]);
    secondCached.release();
  });
}

VideoMediaSource _media(String url, List<String> fallbacks, String digest) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(url, fallbackUrls: fallbacks),
    digest,
  );
}

const _firstDigest =
    '4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a';
const _secondDigest =
    'dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986';
