import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('digging chains through older pages until the buffer refills',
      () async {
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ])
      ..olderFeedPages.add([samplePost(id: 'older-0')])
      ..olderFeedPages.add([
        for (var index = 1; index < 11; index += 1) samplePost(id: 'older-$index'),
      ])
      ..olderFeedPages.add([samplePost(id: 'unreached')]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(11);
    await pumpEventQueue();

    // One swipe near the end refills the whole buffer without more swipes.
    final state = cubit.state as FeedLoaded;
    expect(state.posts, hasLength(23));
    expect(repository.olderFeedRequests, hasLength(2));
  });
}
