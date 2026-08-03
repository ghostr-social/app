import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('evicts an older cache entry to adopt a native blob', () async {
    final root = await Directory.systemTemp.createTemp('native-eviction-');
    addTearDown(() => root.delete(recursive: true));
    final cache = Directory('${root.path}/cache')..createSync();
    final old = File('${cache.path}/old.video')..writeAsBytesSync([9]);
    old.setLastModifiedSync(DateTime.utc(2026));
    final native = File('${root.path}/native.mp4')..writeAsBytesSync([1, 2]);
    const url = 'https://media.test/video.mp4';
    final store = FileVideoCacheStore(
      directoryProvider: () async => cache,
      downloader: FakeVideoFileDownloader(const {}),
      maxBytes: 2,
    );
    final media = VideoMediaSource.importable(native.path, remoteUrl: url);

    final lease = await store.acquire(media);

    expect(lease, isNotNull);
    expect(old.existsSync(), isFalse);
    expect(File(lease!.media.localPath!).readAsBytesSync(), [1, 2]);
    lease.release();
  });
}
