import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('rejects and deletes a corrupted verified cache after restart',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const url = 'https://media.test/video.mp4';
    final media = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote(url),
      '4bf5122f344554c53bde2ebb8cd2b7e3d'
      '1600ad631c385a5d7cce23c7785459a',
    );
    final downloader = FakeVideoFileDownloader(const {
      url: [1]
    });
    final store = _store(directory, downloader);
    final cached = (await store.acquire(media))!;
    final file = File(cached.media.localPath!);
    cached.release();
    await file.writeAsBytes(const [2], flush: true);

    final failing = FakeVideoFileDownloader(const {}, error: StateError('x'));

    await expectLater(
      _store(directory, failing).acquire(media),
      throwsA(isA<AppFailure>()),
    );
    expect(await file.exists(), isFalse);
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
