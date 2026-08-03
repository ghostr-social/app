import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('rejects a non-positive concurrent transfer limit', () {
    expect(
      () => FileVideoCacheStore(
        directoryProvider: () => throw StateError('unused'),
        downloader: FakeVideoFileDownloader(const {}),
        maxBytes: 10,
        maxConcurrentTransfers: 0,
      ),
      throwsRangeError,
    );
  });
}
