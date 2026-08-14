import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_backfill_retry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test(
    'three dry pages back off before automatically finding a fresh page',
    () {
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
              ..olderFeedPages.add([samplePost(id: 'older-0')]);
        final cubit = FeedCubit(
          FeedDependencies(feed: repository, engagement: repository),
          backfillRetry: FeedBackfillRetry(
            delays: const [Duration(seconds: 1)],
          ),
        );
        cubit.load();
        clock.flushMicrotasks();

        cubit.pageChanged(10);
        clock.flushMicrotasks();
        expect(repository.olderFeedRequests, hasLength(3));
        expect((cubit.state as FeedLoaded).posts, hasLength(12));

        clock.elapse(const Duration(milliseconds: 999));
        expect(repository.olderFeedRequests, hasLength(3));
        clock.elapse(const Duration(milliseconds: 1));
        clock.flushMicrotasks();
        expect(repository.olderFeedRequests, hasLength(4));
        expect((cubit.state as FeedLoaded).posts, hasLength(13));
        cubit.close();
        clock.flushMicrotasks();
      });
    },
  );
}
