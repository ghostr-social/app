import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/profile_video_feed_repository.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a profile feed hides the shelf of a blocked creator', () async {
    final creator = sampleCreator();
    final details = sampleProfileDetails(
      profile: creator,
      posts: [samplePost(creator: creator)],
    );
    final repository = FakeVideoCatalogRepository(
      forYouFeed: const [],
      feed: FakeFeedScenario(profiles: {creator.id: details}),
    );
    repository.blockedProfiles.add(creator.id);
    final feed = ProfileVideoFeedRepository(
      profile: repository,
      viewer: sampleCreator(id: 'viewer'),
      creatorId: creator.id,
    );

    expect(await feed.loadFeed(FeedKind.forYou), isEmpty);
  });
}
