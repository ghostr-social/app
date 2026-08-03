import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('rejects a video that alone exceeds the configured byte budget',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final remote = VideoMediaSource.remote('https://media.test/large.mp4');
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader({
        remote.debugLabel: [1, 2, 3, 4]
      }),
      maxBytes: 3,
    );

    expect(await store.acquire(remote), isNull);
    expect(directory.listSync().whereType<File>(), isEmpty);
  });
}
