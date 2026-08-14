import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('an empty loading snapshot does not erase visible posts', () async {
    final updates = ControllableVideoFeedUpdates();
    final feed = ScriptedFeedRepository(
      loads: [
        [samplePost()],
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(
          delivery: FeedDeliveryDependencies(updates: updates),
        ),
      ),
    );
    addTearDown(cubit.close);
    addTearDown(updates.close);
    await cubit.load();

    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.loading,
        hasPosts: false,
      ),
    );
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoaded>());
    expect(feed.loadExclusions, [true]);
  });
}
