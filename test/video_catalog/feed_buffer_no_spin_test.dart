import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a page of duplicates pauses digging until the next swipe', () async {
    final duplicate = samplePost(id: 'post-0');
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ])
      ..olderFeedPages.add([duplicate])
      ..olderFeedPages.add([samplePost(id: 'older-0')]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(10);
    await pumpEventQueue();
    expect(repository.olderFeedRequests, hasLength(1));
    expect((cubit.state as FeedLoaded).posts, hasLength(12));

    cubit.pageChanged(11);
    await pumpEventQueue();
    expect(repository.olderFeedRequests, hasLength(2));
    expect((cubit.state as FeedLoaded).posts, hasLength(13));
  });
}
