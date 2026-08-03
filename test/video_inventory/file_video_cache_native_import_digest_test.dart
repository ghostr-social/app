import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('rejects a native blob that mismatches the canonical digest', () async {
    final root = await Directory.systemTemp.createTemp('native-digest');
    addTearDown(() => root.delete(recursive: true));
    final native = File('${root.path}/native.mp4');
    await native.writeAsBytes([1, 2, 3]);
    const url = 'https://media.example/video.mp4';
    final canonicalBytes = [4, 5, 6];
    final downloader = FakeVideoFileDownloader({url: canonicalBytes});
    final store = FileVideoCacheStore(
      directoryProvider: () async => Directory('${root.path}/dart-cache'),
      downloader: downloader,
      maxBytes: 64,
    );
    final source = VideoMediaSource.withExpectedSha256(
      VideoMediaSource.importable(native.path, remoteUrl: url),
      sha256.convert(canonicalBytes).toString(),
    );

    final lease = await store.acquire(source);

    expect(downloader.attemptedUrls, [url]);
    expect(await File(lease!.media.localPath!).readAsBytes(), canonicalBytes);
    lease.release();
  });
}
