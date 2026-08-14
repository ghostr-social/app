import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';

import '../support/fakes.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';
import '../support/nostr_video_post_fixture.dart';
import '../support/sample_data.dart';

void main() {
  test('keeps the exact revision named by a specific repost', () async {
    final stale = _revision(secondTestEventId, DateTime.utc(2026, 1), 'stale');
    final current = _revision(testEventId, DateTime.utc(2026, 2), 'current');
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([_specificRepost(stale)]),
    );
    await harness.localStore.savePublishedPosts([current]);

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;

    expect(post.id, stale.id);
    expect(post.caption, 'stale');
    expect(post.repost?.eventId.value, publishedTestEventId);
  });
}

VideoPost _specificRepost(VideoPost post) {
  return VideoPost(
    identity: VideoPostIdentity(
      id: post.id,
      creator: post.creator,
      nostrReference: post.nostrReference,
      repost: VideoRepostAttribution(
        eventId: NostrEventId.parse(publishedTestEventId),
        reposter: sampleCreator(id: 'reposter'),
        repostedAt: DateTime.utc(2026, 3),
        target: VideoRepostTarget.specificEvent,
      ),
    ),
    content: post.content,
    metrics: post.metrics,
  );
}

VideoPost _revision(String eventId, DateTime publishedAt, String caption) {
  final post = nostrVideoPost(
    NostrVideoPostFixture(
      eventId: eventId,
      mediaId: 'same-video',
      text: NostrVideoTextFixture(caption: caption),
    ),
  );
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
