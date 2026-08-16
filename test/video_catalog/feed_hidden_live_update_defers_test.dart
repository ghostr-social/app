import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('a hidden feed defers live reconciliation until it returns', () async {
    final initial = samplePost(id: 'initial');
    final fresh = samplePost(id: 'fresh');
    final updates = ControllableVideoFeedUpdates();
    final feed = ScriptedFeedRepository(
      loads: [
        [initial],
        [initial, fresh],
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: const []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();

    cubit.surfaceVisibilityChanged(false);
    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.settled,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();
    expect(feed.loadCalls, 1);

    cubit.surfaceVisibilityChanged(true);
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(feed.loadCalls, 2);
    expect(feed.loadExclusions, [true, false]);
    expect(loaded.posts.map((post) => post.id.value), ['initial', 'fresh']);
  });
}
