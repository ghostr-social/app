import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';
import 'package:ghostr/platform/media/video_cache_media_files.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('cleans a partial when completed cache installation fails', () async {
    final directory = await Directory.systemTemp.createTemp('cache-install-');
    addTearDown(() => directory.delete(recursive: true));
    final media = VideoMediaSource.remote('https://media.test/video.mp4');
    await Directory(completedVideoCachePath(directory, media)).create();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader({
        media.remoteUrl!: [1]
      }),
      maxBytes: 10,
    );

    await expectLater(store.acquire(media), throwsA(isA<AppFailure>()));

    expect(
      directory.listSync().whereType<File>(),
      isEmpty,
    );
  });
}
