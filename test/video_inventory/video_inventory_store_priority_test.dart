import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/data/smart_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_cache_priority.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/scoped_video_media.dart';

void main() {
  test('starts foreground work before queued prefetch work', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _PriorityDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 40,
      maxConcurrentTransfers: 2,
    );
    final inventory = SmartVideoInventory(
      store: store,
      maxParallelDownloads: 2,
      maxPreparedVideos: 4,
    );
    final media = List.generate(
      4,
      (index) => scopedVideoMedia('https://media.test/$index.mp4'),
    );
    inventory.prepare(media);
    await downloader.twoStarted.future;

    final active = inventory.acquire(
      media.last,
      VideoCachePriority.foreground,
    );
    downloader.release(media.first.remoteUrl!);
    await downloader.foregroundStarted.future;

    expect(downloader.startedUrls, [
      media[0].remoteUrl,
      media[1].remoteUrl,
      media[3].remoteUrl,
    ]);

    downloader.releaseAll();
    (await active)?.release();
    (await inventory.acquire(
      media[2],
      VideoCachePriority.foreground,
    ))
        ?.release();
  });
}

class _PriorityDownloader implements VideoFileDownloader {
  final twoStarted = Completer<void>();
  final foregroundStarted = Completer<void>();
  final List<String> startedUrls = [];
  final Map<String, Completer<void>> _gates = {};
  bool _releaseFutureStarts = false;

  void release(String url) => _gates[url]!.complete();

  void releaseAll() {
    _releaseFutureStarts = true;
    for (final gate in _gates.values.where((gate) => !gate.isCompleted)) {
      gate.complete();
    }
  }

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    final url = source.toString();
    startedUrls.add(url);
    final gate = _gates[url] = Completer<void>();
    if (startedUrls.length == 2) twoStarted.complete();
    if (source.path.endsWith('/3.mp4')) foregroundStarted.complete();
    if (_releaseFutureStarts) gate.complete();
    await gate.future;
    await File(destinationPath).writeAsBytes([1]);
  }
}
