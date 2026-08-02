import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('removes partial files after a download failure', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader({}, error: StateError('offline')),
      maxBytes: 10,
    );
    final remote = VideoMediaSource.remote('https://media.test/video.mp4');

    await expectLater(store.download(remote), throwsA(isA<AppFailure>()));

    expect(directory.listSync(), isEmpty);
  });
}
