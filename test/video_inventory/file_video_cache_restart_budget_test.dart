import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('enforces a reduced budget and removes partials after restart',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final oldMedia = VideoMediaSource.remote('https://media.test/old.mp4');
    final newMedia = VideoMediaSource.remote('https://media.test/new.mp4');
    final downloader = FakeVideoFileDownloader({
      oldMedia.debugLabel: [1, 2, 3, 4],
      newMedia.debugLabel: [5, 6, 7, 8],
    });
    final first = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final oldCached = (await first.acquire(oldMedia))!;
    final newCached = (await first.acquire(newMedia))!;
    final oldPath = oldCached.media.localPath!;
    final newPath = newCached.media.localPath!;
    oldCached.release();
    newCached.release();
    await File(oldPath).setLastModified(DateTime(2000));
    final partial = File('$newPath.partial');
    await partial.writeAsBytes([9]);

    final restarted = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 4,
    );
    await restarted.initialize();
    final restored = (await restarted.acquire(newMedia))!;

    expect(restored.media.localPath, newPath);
    expect(await File(oldPath).exists(), isFalse);
    expect(await partial.exists(), isFalse);
    restored.release();
  });
}
