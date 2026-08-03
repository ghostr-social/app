import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('an exhausted feed stops requesting older pages', () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ])
      ..olderFeedPages.add([samplePost(id: 'older-0')]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(8);
    await pumpEventQueue();
    cubit.pageChanged(9);
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).posts, hasLength(13));
    expect(repository.olderFeedRequests, hasLength(1));
  });
}
