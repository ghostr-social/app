import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('tracks the active page in a loaded feed', () async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'one'), samplePost(id: 'two')],
    );
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(1);

    expect((cubit.state as FeedLoaded).activeIndex, 1);
  });
}
