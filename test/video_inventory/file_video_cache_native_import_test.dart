import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('copies a warm native blob into the leased Dart cache', () async {
    final root = await Directory.systemTemp.createTemp('native-import');
    addTearDown(() => root.delete(recursive: true));
    final native = File('${root.path}/native.mp4');
    final bytes = [1, 2, 3, 4];
    await native.writeAsBytes(bytes);
    const url = 'https://media.example/video.mp4';
    final downloader = FakeVideoFileDownloader({
      url: [9, 9]
    });
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
    final managed = File(lease!.media.localPath!);
    expect(downloader.attemptedUrls, isEmpty);
    expect(managed.path, isNot(native.path));
    expect(await managed.readAsBytes(), bytes);

    await native.delete();
    expect(await managed.readAsBytes(), bytes);
    lease.release();
  });
}
