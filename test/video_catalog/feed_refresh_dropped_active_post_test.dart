import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a refresh that drops the active post clamps to the nearest one',
      () async {
    final posts = [
      samplePost(id: 'post-0'),
      samplePost(id: 'post-1'),
      samplePost(id: 'post-2'),
    ];
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();
    cubit.pageChanged(2);
    repository.forYouFeed.removeRange(1, 3);

    await cubit.refresh();

    final state = cubit.state as FeedLoaded;
    expect(state.posts.single.id.value, 'post-0');
    expect(state.activeIndex, 0);
  });
}
