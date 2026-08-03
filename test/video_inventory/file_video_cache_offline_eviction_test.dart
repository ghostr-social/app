import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('keeps an existing entry when a full-cache download is offline',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final existing = File('${directory.path}/existing.video');
    await existing.writeAsBytes([9]);
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader(
        const {},
        error: StateError('offline'),
      ),
      maxBytes: 1,
    );

    await expectLater(
      store.acquire(VideoMediaSource.remote('https://media.test/new.mp4')),
      throwsA(isA<AppFailure>()),
    );

    expect(await existing.readAsBytes(), [9]);
  });
}
