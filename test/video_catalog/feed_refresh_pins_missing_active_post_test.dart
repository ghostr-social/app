import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'a passive refresh pins an active post missing from its snapshot',
    () async {
      final posts = [
        samplePost(id: 'post-0'),
        samplePost(id: 'post-1'),
        samplePost(id: 'post-2'),
      ];
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(feed: repository, engagement: repository),
      );
      addTearDown(cubit.close);
      await cubit.load();
      cubit.pageChanged(2);
      repository.forYouFeed.removeRange(1, 3);

      await cubit.refresh();

      final state = cubit.state as FeedLoaded;
      expect(state.posts.map((post) => post.id.value), [
        'post-0',
        'post-1',
        'post-2',
      ]);
      expect(state.activeIndex, 2);
    },
  );
}
