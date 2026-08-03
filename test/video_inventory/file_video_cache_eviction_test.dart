import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('evicts the least recently used video to honor the byte budget',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = FakeVideoFileDownloader({
      'https://media.test/old.mp4': [1, 2, 3, 4],
      'https://media.test/new.mp4': [5, 6, 7, 8],
    });
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 6,
    );
    final oldMedia = VideoMediaSource.remote('https://media.test/old.mp4');
    final newMedia = VideoMediaSource.remote('https://media.test/new.mp4');

    final oldCached = (await store.acquire(oldMedia))!;
    final oldFile = File(oldCached.media.localPath!);
    await oldFile.setLastModified(DateTime(2000));
    oldCached.release();
    final newCached = (await store.acquire(newMedia))!;

    expect(newCached.media.isLocal, isTrue);
    expect(await oldFile.exists(), isFalse);
    expect(await File(newCached.media.localPath!).exists(), isTrue);
    newCached.release();
  });
}
