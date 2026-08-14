import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a delayed backfill retry rechecks the viewer buffer', () {
    fakeAsync((clock) {
      final duplicate = samplePost(id: 'post-0');
      final repository =
          FakeVideoCatalogRepository(
              forYouFeed: [
                for (var index = 0; index < 12; index += 1)
                  samplePost(id: 'post-$index'),
              ],
            )
            ..olderFeedPages.add([duplicate])
            ..olderFeedPages.add(const [])
            ..olderFeedPages.add([duplicate])
            ..olderFeedPages.add([samplePost(id: 'unneeded')]);
      final cubit = FeedCubit(
        FeedDependencies(feed: repository, engagement: repository),
        backfillRetry: FeedBackfillRetry(delays: const [Duration(seconds: 1)]),
      );
      cubit.load();
      clock.flushMicrotasks();

      cubit.pageChanged(10);
      clock.flushMicrotasks();
      expect(repository.olderFeedRequests, hasLength(3));
      cubit.pageChanged(0);

      clock.elapse(const Duration(seconds: 1));
      clock.flushMicrotasks();
      expect(repository.olderFeedRequests, hasLength(3));
      cubit.close();
      clock.flushMicrotasks();
    });
  });
}
