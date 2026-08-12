import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a profile feed reports its past exhausted', () async {
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(
            profile: creator,
            posts: [samplePost(id: 'clip-1', creator: creator)],
          ),
        },
      ),
    );
    final feed = ProfileVideoFeedRepository(
      profile: repository,
      viewer: sampleCreator(id: 'viewer-1'),
      creatorId: creator.id,
    );

    final page = await feed.loadOlderFeed(
      FeedKind.forYou,
      olderThan: DateTime(2026, 3, 12),
    );

    expect(page.posts, isEmpty);
    expect(page.hasMore, isFalse);
  });
}
