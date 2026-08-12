import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a profile feed serves the creator shelf for any feed kind', () async {
    final creator = sampleCreator();
    final shelf = [
      samplePost(id: 'clip-1', creator: creator),
      samplePost(id: 'clip-2', creator: creator),
    ];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'other')],
      feed: FakeFeedScenario(
        profiles: {
          creator.id: sampleProfileDetails(profile: creator, posts: shelf),
        },
      ),
    );
    final feed = ProfileVideoFeedRepository(
      profile: repository,
      viewer: sampleCreator(id: 'viewer-1'),
      creatorId: creator.id,
    );

    expect(await feed.loadFeed(FeedKind.forYou, excludeWatched: true), shelf);
    expect(await feed.loadFeed(FeedKind.following), shelf);
  });
}
