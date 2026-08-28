import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_aware_video_feed_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a rescue remains reachable through the next live refresh', () async {
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final history = FakeWatchHistoryRepository();
    final reporter = RecordingFailureReporter();
    final delivery = ControlledVideoDeliveryUpdates();
    final cubit = FeedCubit(
      FeedDependencies(
        feed: WatchAwareVideoFeedRepository(
          feed: source,
          history: history,
          failureReporter: reporter,
        ),
        engagement: source,
        optional: FeedOptionalDependencies(
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: reporter,
            ),
          ),
          delivery: FeedDeliveryDependencies(deliveryUpdates: delivery),
        ),
      ),
    );
    addTearDown(() async {
      await Future.wait([cubit.close(), delivery.close()]);
    });
    await cubit.load();
    delivery.publish(posts[1], phase: VideoDeliveryPhase.preparing);
    delivery.publish(posts[2], phase: VideoDeliveryPhase.startable);
    cubit.pageChanged(1);
    await pumpEventQueue();
    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p2');

    await cubit.refresh();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts.map((post) => post.id.value), ['p0', 'p1', 'p2']);
    expect(loaded.roster.active.id.value, 'p2');
  });
}
