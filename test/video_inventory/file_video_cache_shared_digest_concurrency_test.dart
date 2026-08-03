import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('coalesces queued source sets that share a verified digest', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _BlockingDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = _media('https://one.test/video.mp4');
    final second = _media('https://two.test/video.mp4');

    final firstResult = store.acquire(first);
    await downloader.started.future;
    final secondResult = store.acquire(second);
    downloader.release.complete();

    final firstLease = (await firstResult)!;
    final secondLease = (await secondResult)!;
    expect(firstLease.media.isLocal, isTrue);
    expect(secondLease.media.isLocal, isTrue);
    expect(downloader.attemptedUrls, ['https://one.test/video.mp4']);
    firstLease.release();
    secondLease.release();
  });
}

VideoMediaSource _media(String url) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(url),
    'dbc1b4c900ffe48d575b5da5c6380401'
    '25f65db0fe3e24494b76ea986457d986',
  );
}

class _BlockingDownloader implements VideoFileDownloader {
  final started = Completer<void>();
  final release = Completer<void>();
  final List<String> attemptedUrls = [];

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    attemptedUrls.add(source.toString());
    if (attemptedUrls.length == 1) {
      started.complete();
      await release.future;
    }
    await File(destinationPath).writeAsBytes([2]);
  }
}
