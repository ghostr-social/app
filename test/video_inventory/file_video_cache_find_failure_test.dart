import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/file_video_cache_store.dart';

import '../support/fake_video_file_downloader.dart';

void main() {
  test('translates a cache directory lookup failure', () async {
    final store = FileVideoCacheStore(
      directoryProvider: () => throw StateError('disk unavailable'),
      downloader: FakeVideoFileDownloader(const {}),
      maxBytes: 10,
    );

    await expectLater(
      store.acquire(VideoMediaSource.remote('https://media.test/video.mp4')),
      throwsA(isA<AppFailure>()),
    );
  });
}
