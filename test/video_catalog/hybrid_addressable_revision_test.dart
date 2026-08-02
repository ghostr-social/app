import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';

void main() {
  test('merges local and relay revisions by addressable coordinate', () async {
    final publishedAt = DateTime.utc(2026);
    final stale = _revision(secondTestEventId, publishedAt, 'stale');
    final current = _revision(testEventId, publishedAt, 'current');
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([current]),
    );
    await harness.localStore.savePublishedPosts([stale]);

    final posts = await harness.feed.loadFeed(FeedKind.forYou);

    expect(posts, hasLength(1));
    expect(posts.single.id, current.id);
    expect(posts.single.caption, 'current');
  });
}

VideoPost _revision(String eventId, DateTime publishedAt, String caption) {
  final post = nostrVideoPost(NostrVideoPostFixture(
    eventId: eventId,
    mediaId: 'same-video',
    text: NostrVideoTextFixture(caption: caption),
  ));
  return VideoPost(
    identity: post.identity,
    content: VideoPostContent(
      caption: post.caption,
      songName: post.songName,
      media: post.media,
      publishedAt: publishedAt,
    ),
    metrics: post.metrics,
  );
}
