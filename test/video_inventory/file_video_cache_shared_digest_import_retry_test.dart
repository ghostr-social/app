import 'dart:async';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

void main() {
  test('a joining digest request retries its distinct native import', () async {
    final root = await Directory.systemTemp.createTemp('native-flight');
    addTearDown(() => root.delete(recursive: true));
    final bytes = [2, 3, 5];
    final validImport = File('${root.path}/valid.mp4');
    await validImport.writeAsBytes(bytes);
    final downloader = _OfflineDownloader();
    final store = FileVideoCacheStore(
      directoryProvider: () async => Directory('${root.path}/cache'),
      downloader: downloader,
      maxBytes: 64,
    );
    final digest = sha256.convert(bytes).toString();
    final first = store.acquire(_media('${root.path}/missing.mp4', digest));
    await downloader.started.future;
    final second = store.acquire(_media(validImport.path, digest));

    downloader.release.complete();

    await expectLater(first, throwsA(isA<AppFailure>()));
    final lease = (await second)!;
    expect(await File(lease.media.localPath!).readAsBytes(), bytes);
    expect(downloader.attempts, 1);
    lease.release();
  });
}

VideoMediaSource _media(String importPath, String digest) {
  return VideoMediaSource.withExpectedSha256(
    VideoMediaSource.importable(
      importPath,
      remoteUrl: 'https://offline.test/video.mp4',
    ),
    digest,
  );
}

class _OfflineDownloader implements VideoFileDownloader {
  final started = Completer<void>();
  final release = Completer<void>();
  int attempts = 0;

  @override
  Future<void> download(
    Uri source,
    String destinationPath, {
    required int maxBytes,
    Duration? totalTimeout,
  }) async {
    attempts += 1;
    started.complete();
    await release.future;
    throw StateError('offline');
  }
}
