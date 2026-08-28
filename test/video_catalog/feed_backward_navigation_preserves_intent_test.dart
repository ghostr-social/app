import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controlled_video_delivery_updates.dart';
import '../support/fake_feed_focus_port.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('backward history traversal preserves the intended video', () async {
    final updates = ControlledVideoDeliveryUpdates();
    final focus = FakeFeedFocusPort();
    final posts = List.generate(3, (index) => samplePost(id: 'p$index'));
    final repository = FakeVideoCatalogRepository(forYouFeed: posts);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: repository,
        engagement: repository,
        optional: FeedOptionalDependencies(
          focus: focus,
          delivery: FeedDeliveryDependencies(deliveryUpdates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();
    cubit.pageChanged(1);
    await pumpEventQueue();
    cubit.pageChanged(2);
    await pumpEventQueue();
    updates.publish(posts[1], phase: VideoDeliveryPhase.preparing);
    updates.publish(posts[0], phase: VideoDeliveryPhase.startable);

    cubit.pageChanged(1);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.roster.active.id.value, 'p1');
    expect(focus.focuses.last.cause, FeedFocusCause.userNavigation);
    expect(
      focus.focuses.where(
        (item) => item.cause == FeedFocusCause.transportRescue,
      ),
      isEmpty,
    );
  });
}
