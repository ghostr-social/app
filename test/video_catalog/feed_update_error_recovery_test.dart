import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_update_retry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/scripted_feed_repository.dart';
import '../support/scripted_video_feed_updates.dart';

void main() {
  test('a failed update stream reconnects and consumes new content', () async {
    final first = StreamController<VideoFeedUpdate>();
    final second = StreamController<VideoFeedUpdate>();
    addTearDown(first.close);
    addTearDown(second.close);
    final updates = ScriptedVideoFeedUpdates([
      () => first.stream,
      () => second.stream,
    ]);
    final feed = ScriptedFeedRepository(
      loads: [
        [samplePost(id: 'initial')],
        [samplePost(id: 'initial', caption: 'recovered')],
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
      updateRetry: FeedUpdateRetry(delays: const [Duration.zero]),
    );
    addTearDown(cubit.close);
    await cubit.load();

    first.addError(StateError('offline'));
    await pumpEventQueue();
    expect(updates.watchCalls, 2);
    second.add(_update());
    await pumpEventQueue();

    expect((cubit.state as FeedLoaded).posts.single.caption, 'recovered');
  });
}

VideoFeedUpdate _update() => VideoFeedUpdate(
  revision: BigInt.one,
  phase: VideoFeedUpdatePhase.loading,
  hasPosts: true,
);
