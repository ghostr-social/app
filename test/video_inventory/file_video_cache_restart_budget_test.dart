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
    final oldCached = await first.download(oldMedia);
    final newCached = await first.download(newMedia);
    await File(oldCached!.localPath!).setLastModified(DateTime(2000));
    final partial = File('${newCached!.localPath!}.partial');
    await partial.writeAsBytes([9]);

    final restarted = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 4,
    );
    final restored = await restarted.find(newMedia);

    expect(restored?.localPath, newCached.localPath);
    expect(await File(oldCached.localPath!).exists(), isFalse);
    expect(await partial.exists(), isFalse);
  });
}
