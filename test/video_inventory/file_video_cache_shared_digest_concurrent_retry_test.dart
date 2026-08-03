import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('a joining digest mirror retries after the shared source fails',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final downloader = _FailingPrimaryDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: downloader,
      maxBytes: 10,
    );
    final first = store.acquire(_media('https://one.test/video.mp4'));
    await downloader.primaryStarted.future;
    final second = store.acquire(_media('https://two.test/video.mp4'));

    downloader.releasePrimary.complete();

    await expectLater(first, throwsA(isA<AppFailure>()));
    final lease = (await second)!;
    expect(downloader.attemptedUrls, [
      'https://one.test/video.mp4',
      'https://two.test/video.mp4',
    ]);
    lease.release();
  });
}

VideoMediaSource _media(String url) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.remote(url),
    'dbc1b4c900ffe48d575b5da5c6380401'
    '25f65db0fe3e24494b76ea986457d986',
  );
}

class _FailingPrimaryDownloader implements VideoFileDownloader {
  final primaryStarted = Completer<void>();
  final releasePrimary = Completer<void>();
  final List<String> attemptedUrls = [];

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    attemptedUrls.add(source.toString());
    if (source.host == 'one.test') {
      primaryStarted.complete();
      await releasePrimary.future;
      throw StateError('offline');
    }
    await File(destinationPath).writeAsBytes([2]);
  }
}
