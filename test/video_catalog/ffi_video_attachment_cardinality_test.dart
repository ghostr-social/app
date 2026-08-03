import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('emits one post when one Nostr event has multiple media rows', () async {
    final canonical = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'dance',
    ));
    final source = FfiVideoRemoteSource(
      snapshotLoader: () => [canonical],
      loader: () async => [
        ffiVideo(
          id: 'video-1',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(
            mediaUrl: 'https://source.example/dance.mp4',
          ),
        ),
        ffiVideo(
          id: 'video-2',
          user: const FfiUserData(),
          options: const FfiVideoFixtureOptions(
            mediaUrl: 'https://source.example/video-2.mp4',
            localPath: '/cache/video-2.mp4',
          ),
        ),
      ],
    );

    final posts = await source.loadRemoteFeed();

    expect(posts, hasLength(1));
    expect(posts.single.id, canonical.id);
  });
}
