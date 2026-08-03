import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('rejects and removes bytes that do not match the advertised digest',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const url = 'https://media.test/video.mp4';
    final media = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote(url),
      'e3b0c44298fc1c149afbf4c8996fb924'
      '27ae41e4649b934ca495991b7852b855',
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader(const {
        url: [1, 2, 3]
      }),
      maxBytes: 10,
    );

    await expectLater(store.acquire(media), throwsA(isA<AppFailure>()));

    expect(directory.listSync(), isEmpty);
  });
}
