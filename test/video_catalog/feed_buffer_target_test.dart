import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('older pages load whenever fewer than ten videos remain ahead',
      () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 20; index += 1) samplePost(id: 'post-$index'),
    ])
      ..olderFeedPages.add([samplePost(id: 'older-0')]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();
    expect(repository.olderFeedRequests, isEmpty);

    cubit.pageChanged(9);
    await pumpEventQueue();
    expect(repository.olderFeedRequests, isEmpty);

    cubit.pageChanged(10);
    await pumpEventQueue();
    expect(repository.olderFeedRequests, hasLength(1));
  });
}
