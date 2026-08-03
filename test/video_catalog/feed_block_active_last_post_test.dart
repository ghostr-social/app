import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('blocking while on the last video falls back to the nearest one',
      () async {
    final kept = sampleCreator(id: 'creator-kept');
    final blocked = sampleCreator(id: 'creator-blocked');
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'post-1', creator: kept),
      samplePost(id: 'post-2', creator: blocked),
      samplePost(id: 'post-3', creator: blocked),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      social: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();
    cubit.pageChanged(2);

    await cubit.blockCreator(repository.forYouFeed.last);

    final state = cubit.state as FeedLoaded;
    expect(state.posts.single.creator.id, kept.id);
    expect(state.activeIndex, 0);
  });
}
