import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import '../support/controllable_video_feed_updates.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';

void main() {
  test('a passive revision replaces hunting immediately', () async {
    final updates = ControllableVideoFeedUpdates();
    final feed = ScriptedFeedRepository(
      loads: [
        const [],
        [samplePost(id: 'found')],
      ],
    );
    final cubit = FeedCubit(
      FeedDependencies(
        feed: feed,
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
    );

    await cubit.load();
    expect(cubit.state, isA<FeedEmpty>());

    updates.add(
      VideoFeedUpdate(
        revision: BigInt.one,
        phase: VideoFeedUpdatePhase.loading,
        hasPosts: true,
      ),
    );
    await pumpEventQueue();

    expect(cubit.state, isA<FeedLoaded>());
    expect(feed.loadCalls, 2);
    expect(updates.watchedKinds, [FeedKind.forYou]);
    await cubit.close();
    expect(updates.cancellations, 1);
    await updates.close();
  });
}
