import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('serializes partial downloads under one aggregate byte budget',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _CoordinatedDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 20,
    );
    final first = VideoMediaSource.remote('https://media.test/first.mp4');
    final second = VideoMediaSource.remote('https://media.test/second.mp4');

    final firstResult = store.download(first);
    await downloader.firstPartialWritten.future;
    final secondResult = store.download(second);
    await Future<void>.delayed(Duration.zero);
    expect(downloader.startedCount, 1);
    downloader.releaseFirst.complete();

    expect((await firstResult)?.isLocal, isTrue);
    expect((await secondResult)?.isLocal, isTrue);
  });
}

class _CoordinatedDownloader implements VideoFileDownloader {
  final firstPartialWritten = Completer<void>();
  final releaseFirst = Completer<void>();
  int startedCount = 0;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
  }) async {
    startedCount += 1;
    await File(destinationPath).writeAsBytes([1, 2, 3]);
    if (!source.path.endsWith('first.mp4')) return;
    firstPartialWritten.complete();
    await releaseFirst.future;
  }
}
