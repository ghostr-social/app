import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('never exceeds the configured concurrent transfer limit', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _LimitedDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 30,
      maxConcurrentTransfers: 2,
    );
    final results = List.generate(
      3,
      (index) => store.acquire(
        VideoMediaSource.remote('https://media.test/$index.mp4'),
      ),
    );

    await downloader.twoStarted.future;

    expect(downloader.started, 2);
    expect(downloader.maximumActive, 2);
    downloader.release.complete();
    for (final result in results) {
      (await result)?.release();
    }
  });
}

class _LimitedDownloader implements VideoFileDownloader {
  final twoStarted = Completer<void>();
  final release = Completer<void>();
  int started = 0;
  int active = 0;
  int maximumActive = 0;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    started += 1;
    active += 1;
    if (active > maximumActive) maximumActive = active;
    if (started == 2) twoStarted.complete();
    await release.future;
    active -= 1;
    await File(destinationPath).writeAsBytes([1]);
  }
}
