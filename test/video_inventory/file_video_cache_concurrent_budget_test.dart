import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('bounds concurrent transfer allowances by the aggregate budget',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _BudgetDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = store.acquire(
      VideoMediaSource.remote('https://media.test/first.mp4'),
    );
    final second = store.acquire(
      VideoMediaSource.remote('https://media.test/second.mp4'),
    );
    addTearDown(() async {
      if (!downloader.release.isCompleted) downloader.release.complete();
      (await first)?.release();
      (await second)?.release();
    });

    await downloader.bothStarted.future;

    expect(
      downloader.activeAllowances.reduce((a, b) => a + b),
      lessThanOrEqualTo(10),
    );
    downloader.release.complete();
    expect((await first)?.media.isLocal, isTrue);
    expect((await second)?.media.isLocal, isTrue);
  });
}

class _BudgetDownloader implements VideoFileDownloader {
  final bothStarted = Completer<void>();
  final release = Completer<void>();
  final List<int> activeAllowances = [];

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    activeAllowances.add(maxBytes);
    if (activeAllowances.length == 2) bothStarted.complete();
    await release.future;
    await File(destinationPath).writeAsBytes([1, 2, 3]);
  }
}
