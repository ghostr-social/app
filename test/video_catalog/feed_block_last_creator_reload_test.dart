import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('blocking the only remaining creator reloads the feed', () async {
    final creator = sampleCreator(id: 'creator-only');
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'post-1', creator: creator),
      samplePost(id: 'post-2', creator: creator),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      optional: FeedOptionalDependencies(social: repository),
    ));
    addTearDown(cubit.close);
    await cubit.load();

    await cubit.blockCreator(repository.forYouFeed.first);

    expect(cubit.state, isA<FeedEmpty>());
    expect(repository.loadFeedExclusions, hasLength(2));
  });
}
