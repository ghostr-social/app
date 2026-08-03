import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a shallow first load tops itself up from older pages', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'post-0'),
      samplePost(id: 'post-1'),
    ])
      ..olderFeedPages.add([samplePost(id: 'older-0')]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);

    await cubit.load();
    await pumpEventQueue();

    final state = cubit.state as FeedLoaded;
    expect(state.posts.map((post) => post.id.value),
        ['post-0', 'post-1', 'older-0']);
    expect(state.activeIndex, 0);
  });
}
