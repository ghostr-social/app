import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('a verified video that disappears from disk is downloaded again',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    const url = 'https://media.test/video.mp4';
    final media = VideoMediaSource.remote(url);
    final downloader = FakeVideoFileDownloader({
      media.debugLabel: [1, 2]
    });
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = (await store.acquire(media))!;
    final file = File(first.media.localPath!);
    first.release();
    await file.delete();

    final second = await store.acquire(media);

    expect(second, isNotNull);
    expect(downloader.attemptedUrls, hasLength(2));
    second!.release();
  });
}
