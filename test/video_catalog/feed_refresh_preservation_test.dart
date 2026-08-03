import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/sample_data.dart';

void main() {
  test('feed refresh preserves the active page', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'one'),
      samplePost(id: 'two'),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();
    cubit.pageChanged(1);

    await cubit.refresh();

    expect((cubit.state as FeedLoaded).activeIndex, 1);
  });
}
