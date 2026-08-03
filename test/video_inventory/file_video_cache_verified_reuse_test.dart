import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('a video verified once this session is served without re-hashing',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const url = 'https://media.test/video.mp4';
    final media = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.remote(url),
      '4bf5122f344554c53bde2ebb8cd2b7e3d'
      '1600ad631c385a5d7cce23c7785459a',
    );
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader(const {
        url: [1]
      }),
      maxBytes: 10,
    );
    final first = (await store.acquire(media))!;
    final file = File(first.media.localPath!);
    first.release();
    // A digest mismatch would reject the file if it were re-hashed.
    await file.writeAsBytes(const [2], flush: true);

    final second = await store.acquire(media);

    expect(second, isNotNull);
    expect(second!.media.localPath, file.path);
    second.release();
  });
}
