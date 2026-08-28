import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('delivery cannot rescue after feed closing starts', () async {
    final history = SecondWriteGatedWatchHistoryRepository();
    final delivery = ControlledVideoDeliveryUpdates();
    final focus = FakeFeedFocusPort();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final source = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: source,
        engagement: source,
        optional: FeedOptionalDependencies(
          focus: focus,
          delivery: FeedDeliveryDependencies(deliveryUpdates: delivery),
          watch: FeedWatchDependencies(
            tracker: WatchHistoryTracker(
              history: history,
              failureReporter: RecordingFailureReporter(),
            ),
          ),
        ),
      ),
    );
    addTearDown(() async {
      if (!history.release.isCompleted) history.release.complete();
      if (!cubit.isClosed) await cubit.close();
      await delivery.close();
    });
    await cubit.load();
    cubit.pageChanged(1);
    await history.started.future;
    final focusCount = focus.focuses.length;

    final closing = cubit.close();
    delivery.publish(posts[2], phase: VideoDeliveryPhase.startable);
    delivery.publish(posts[0], phase: VideoDeliveryPhase.failed);
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).roster.active.id.value, 'p0');
    expect(focus.focuses, hasLength(focusCount));
    history.release.complete();
    await closing;
  });
}
