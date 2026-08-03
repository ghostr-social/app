import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('translates an unexpected cache initialization failure', () async {
    final store = FileVideoCacheStore(
      directoryProvider: () => throw StateError('disk unavailable'),
      downloader: FakeVideoFileDownloader(const {}),
      maxBytes: 10,
    );

    await expectLater(store.initialize(), throwsA(isA<AppFailure>()));
  });
}
