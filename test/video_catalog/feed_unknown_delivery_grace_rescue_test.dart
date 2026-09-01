import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/feed_preparation_updates.dart';
import '../support/player_verified_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('unknown forward delivery rescues after the bounded grace', () {
    fakeAsync((clock) {
      final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
      final repository = FakeVideoCatalogRepository(forYouFeed: posts);
      final preparation = ControlledPlaybackPreparationUpdates();
      final focus = FakeFeedFocusPort();
      final cubit = FeedCubit(
        FeedDependencies(
          feed: repository,
          engagement: repository,
          optional: FeedOptionalDependencies(
            focus: focus,
            delivery: FeedDeliveryDependencies(preparationUpdates: preparation),
          ),
        ),
      );
      cubit.load();
      clock.flushMicrotasks();
      preparation.publish(
        playerVerifiedPlan(posts, currentIndex: 0, readyIndices: [2]),
      );
      clock.flushMicrotasks();

      cubit.pageChanged(1);
      clock.flushMicrotasks();
      expect((cubit.state as FeedLoaded).activeIndex, 1);
      clock.elapse(const Duration(milliseconds: 250));
      clock.flushMicrotasks();

      expect((cubit.state as FeedLoaded).activeIndex, 2);
      expect(focus.focuses.last.cause, FeedFocusCause.transportRescue);
      cubit.close();
      preparation.close();
      clock.flushMicrotasks();
    });
  });
}
