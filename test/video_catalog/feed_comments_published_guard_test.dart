import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores comment publications unless a positive count can be applied',
      () async {
    final post = samplePost();
    final repository = FakeVideoCatalogRepository(forYouFeed: [post]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);

    final loading = cubit.state;
    cubit.commentsPublished(post, 1);
    expect(cubit.state, same(loading));

    await cubit.load();
    final loaded = cubit.state;
    cubit.commentsPublished(post, 0);
    expect(cubit.state, same(loaded));
  });
}
