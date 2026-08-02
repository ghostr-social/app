import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('preserves an app-safe startup maintenance failure', () async {
    const failure = AppFailure('Cache storage unavailable.');
    final store = FileVideoCacheStore(
      directoryProvider: () => throw failure,
      downloader: FakeVideoFileDownloader({}),
      maxBytes: 1,
    );

    final result = store.initialize();

    await expectLater(result, throwsA(same(failure)));
  });
}
