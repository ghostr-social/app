import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';

import '../support/fakes.dart';
import '../support/hybrid_revision_fixture.dart';
import '../support/hybrid_repository_harness.dart';
import '../support/nostr_test_values.dart';

void main() {
  test('combines newest revision with newest repost occurrence', () async {
    final stale = addressableRevision(
      secondTestEventId,
      DateTime.utc(2026, 1),
      'stale',
    );
    final current = addressableRevision(
      testEventId,
      DateTime.utc(2026, 2),
      'current',
    );
    final harness = await buildHybridRepositoryHarness(
      FakeRemoteVideoSource([coordinateRepost(stale)]),
    );
    await harness.localStore.savePublishedPosts([current]);

    final post = (await harness.feed.loadFeed(FeedKind.forYou)).single;

    expect(post.id, current.id);
    expect(post.caption, 'current');
    expect(post.repost?.eventId.value, publishedTestEventId);
    expect(post.feedActivityAt, DateTime.utc(2026, 3));
  });
}
