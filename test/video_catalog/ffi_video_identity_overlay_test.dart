import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('retains cache metadata only for the exact canonical Nostr event',
      () async {
    const sharedUrl = 'https://source.example/shared.mp4';
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'shared',
    )).withMedia(VideoMediaSource.withCacheScope(
      VideoMediaSource.withExpectedSha256(
        VideoMediaSource.remote(sharedUrl),
        'e3b0c44298fc1c149afbf4c8996fb924'
        '27ae41e4649b934ca495991b7852b855',
      ),
      testEventId,
    ));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        _native('other', secondTestEventId, '/cache/other.mp4'),
        _native(
          'e3b0c44298fc1c149afbf4c8996fb924'
              '27ae41e4649b934ca495991b7852b855',
          testEventId,
          '/cache/genuine.mp4',
        ),
      ],
    );

    final post = (await source.loadRemoteFeed())
        .firstWhere((post) => post.id.value == testEventId);

    expect(post.media.localPath, isNull);
    expect(post.media.remoteUrl, 'https://source.example/shared.mp4');
    expect(
      post.media.expectedSha256?.value,
      canonical.media.expectedSha256?.value,
    );
    expect(post.media.cacheScope?.value, canonical.media.cacheScope?.value);
  });
}

FfiVideoDownload _native(String id, String eventId, String path) {
  return ffiVideo(
    id: id,
    user: const FfiUserData(),
    options: FfiVideoFixtureOptions(
      localPath: path,
      mediaUrl: 'https://source.example/shared.mp4',
    ),
    event: ffiNostrEvent(
      eventId: eventId,
      identifier: 'shared',
    ),
  );
}
