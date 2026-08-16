import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/controllable_video_delivery_updates.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a hidden feed pauses delivery rescue until it returns', () {
    fakeAsync((clock) {
      final updates = ControllableVideoDeliveryUpdates();
      final focus = FakeFeedFocusPort();
      final history = FakeWatchHistoryRepository();
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            focus: focus,
            delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
            watch: FeedWatchDependencies(
              tracker: WatchHistoryTracker(
                history: history,
                failureReporter: RecordingFailureReporter(),
              ),
            ),
          ),
        ),
      );
      unawaited(cubit.load());
      clock.flushMicrotasks();
      updates.publish(posts[0], startable: true);
      updates.publish(posts[1], startable: false, etaMilliseconds: 100);
      updates.publish(posts[2], startable: true);
      cubit.pageChanged(1);
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).activeIndex, 1);

      cubit.surfaceVisibilityChanged(false);
      updates.publish(posts[1], startable: false);
      clock.elapse(const Duration(milliseconds: 250));

      final hidden = cubit.state as FeedLoaded;
      expect(hidden.activeIndex, 1);
      expect(history.entries, hasLength(2));

      cubit.surfaceVisibilityChanged(true);
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).activeIndex, 2);
      expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
      expect(history.entries, hasLength(3));
      unawaited(cubit.close());
      unawaited(updates.close());
      clock.flushMicrotasks();
    });
  });
}
