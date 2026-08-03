import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('ignores a stale page index outside the current feed', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final cubit = FeedCubit(
      FeedDependencies(feed: repository, engagement: repository),
    );
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(5);

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.activeIndex, 0);
  });
}
