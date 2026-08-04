import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_engagement.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('only a creator that ends up blocked costs the feed its posts',
      () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: const []);
    final engagement = FeedEngagement(repository, repository);
    final post = samplePost();

    expect(await engagement.block(post), isA<FeedCreatorBlocked>());
    expect(await engagement.block(post), isA<FeedCreatorKept>());
  });

  test('a feed without a social graph cannot block anyone', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: const []);

    final result = await FeedEngagement(repository).block(samplePost());

    expect(result, isA<FeedCreatorKept>());
    expect(repository.blockedProfiles, isEmpty);
  });
}
