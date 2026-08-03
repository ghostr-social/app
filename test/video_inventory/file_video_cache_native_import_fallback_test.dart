import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('downloads remotely when the native import disappears', () async {
    final root = await Directory.systemTemp.createTemp('native-fallback');
    addTearDown(() => root.delete(recursive: true));
    final native = File('${root.path}/native.mp4');
    await native.writeAsBytes([1]);
    await native.delete();
    const url = 'https://media.example/video.mp4';
    final bytes = [5, 6, 7];
    final downloader = FakeVideoFileDownloader({url: bytes});
    final store = FileVideoCacheStore(
      directoryProvider: () async => Directory('${root.path}/dart-cache'),
      downloader: downloader,
      maxBytes: 64,
    );
    final source = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.importable(native.path, remoteUrl: url),
      sha256.convert(bytes).toString(),
    );

    final lease = await store.acquire(source);

    expect(downloader.attemptedUrls, [url]);
    expect(await File(lease!.media.localPath!).readAsBytes(), bytes);
    lease.release();
  });
}
