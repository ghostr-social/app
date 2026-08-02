import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/src/rust/video/video.dart';

import '../support/ffi_video_fixture.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('overlays canonical Nostr posts with native video locations', () async {
    var loadCount = 0;
    final local = nostrVideoPost(const NostrVideoPostFixture(
      eventId: testEventId,
      mediaId: 'local',
    ));
    final remote = nostrVideoPost(const NostrVideoPostFixture(
      eventId: secondTestEventId,
      mediaId: 'remote',
    ));
    final source = FfiVideoRemoteSource(
      gatewayBaseUrl: 'http://127.0.0.1:3000',
      snapshotLoader: () => [local, remote],
      loader: () async {
        loadCount += 1;
        return [
          ffiVideo(
            id: 'local',
            user: const FfiUserData(),
            options: const FfiVideoFixtureOptions(
              localPath: '/cache/local.mp4',
            ),
          ),
          ffiVideo(id: 'remote', user: const FfiUserData()),
          ffiVideo(
            id: 'unknown',
            user: const FfiUserData(),
            event: ffiNostrEvent(eventId: 'not-an-event-id'),
          ),
        ];
      },
    );

    final posts = await source.loadRemoteFeed();

    expect(loadCount, 1);
    expect(posts.map((post) => post.id), [testEventId, secondTestEventId]);
    expect(posts[0].media.localPath, '/cache/local.mp4');
    expect(posts[1].media.remoteUrl, 'https://source.example/remote.mp4');
    expect(
      posts[1].media.fallbackUrls,
      ['http://127.0.0.1:3000/video.mp4?id=remote'],
    );
    expect(posts.every((post) => post.nostrReference != null), isTrue);
  });
}
