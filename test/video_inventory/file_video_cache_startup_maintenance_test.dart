import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('enforces a reduced byte budget during cache startup', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final old = File('${directory.path}/old.video')
      ..writeAsBytesSync([1, 2, 3, 4]);
    final recent = File('${directory.path}/recent.video')
      ..writeAsBytesSync([5, 6, 7, 8]);
    final partial = File('${directory.path}/stale.partial')
      ..writeAsBytesSync([9]);
    await old.setLastModified(DateTime.utc(2000));
    await recent.setLastModified(DateTime.utc(2026));
    final store = FileVideoCacheStore(
      directoryProvider: () async => directory,
      downloader: FakeVideoFileDownloader(const {}),
      maxBytes: 4,
    );

    await store.initialize();

    expect(await old.exists(), isFalse);
    expect(await recent.exists(), isTrue);
    expect(await partial.exists(), isFalse);
  });
}
