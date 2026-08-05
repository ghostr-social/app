import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('a passive refresh preserves the playing video', () async {
    final updates = ControllableVideoFeedUpdates();
    final feed = ScriptedFeedRepository(
      loads: [
        [samplePost(id: 'a'), samplePost(id: 'b')],
        [
          samplePost(id: 'new'),
          samplePost(id: 'a', caption: 'fresh a'),
          samplePost(id: 'b', caption: 'fresh b'),
        ],
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
    );
    final states = <FeedState>[];
    final subscription = cubit.stream.listen(states.add);
    await cubit.load();
    cubit.pageChanged(1);

    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.settled,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();

    final loaded = cubit.state as FeedLoaded;
    expect(loaded.posts[loaded.activeIndex].id.value, 'b');
    expect(loaded.posts[loaded.activeIndex].caption, 'fresh b');
    expect(feed.loadExclusions, [true, false]);
    expect(states.whereType<FeedLoading>(), hasLength(1));
    await subscription.cancel();
    await cubit.close();
    await updates.close();
  });
}
