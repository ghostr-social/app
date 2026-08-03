import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('nearing the end of the feed appends the next older page', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ])
      ..olderFeedPages.add(
        [samplePost(id: 'older-0'), samplePost(id: 'post-11')],
      );
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(7);
    await pumpEventQueue();

    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(13));
    expect(state.posts.last.id.value, 'older-0');
    expect(state.activeIndex, 7);
    expect(
      repository.olderFeedRequests.single,
      samplePost().publishedAt.subtract(const Duration(seconds: 1)),
    );
  });
}
