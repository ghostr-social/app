import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('tries a mirror after the primary bytes fail digest verification',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const primary = 'https://media.test/video.mp4';
    const mirror = 'https://mirror.test/video.mp4';
    final media = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote(primary, fallbackUrls: const [mirror]),
      'dbc1b4c900ffe48d575b5da5c6380401'
      '25f65db0fe3e24494b76ea986457d986',
    );
    final downloader = FakeVideoFileDownloader(const {
      primary: [1],
      mirror: [2],
    });
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );

    final cached = (await store.acquire(media))!;

    expect(downloader.attemptedUrls, [primary, mirror]);
    expect(await File(cached.media.localPath!).readAsBytes(), [2]);
    cached.release();
  });
}
