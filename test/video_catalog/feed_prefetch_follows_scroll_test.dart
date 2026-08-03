import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/feed_media_prefetcher.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_video_inventory.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('scrolling refocuses the media prefetch window', () async {
    final inventory = FakeVideoInventory();
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      for (var index = 0; index < 12; index += 1) samplePost(id: 'post-$index'),
    ]);
    final cubit = FeedCubit(FeedDependencies(
      feed: repository,
      engagement: repository,
      prefetcher: FeedMediaPrefetcher(
        inventory: inventory,
        ahead: 2,
        behind: 1,
      ),
    ));
    addTearDown(cubit.close);
    await cubit.load();

    cubit.pageChanged(6);
    await pumpEventQueue();

    expect(
      inventory.prepared.last.map((media) => media.remoteUrl),
      [
        'https://example.com/video/post-7.mp4',
        'https://example.com/video/post-8.mp4',
        'https://example.com/video/post-5.mp4',
      ],
    );
  });
}
