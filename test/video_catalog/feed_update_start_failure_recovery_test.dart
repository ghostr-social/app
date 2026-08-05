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
  test('a synchronous update-watch failure reconnects automatically', () async {
    final recovered = StreamController<VideoFeedUpdate>();
    addTearDown(recovered.close);
    final updates = ScriptedVideoFeedUpdates([
      () => throw StateError('cannot watch'),
      () => recovered.stream,
    ]);
    final cubit = FeedCubit(
      FeedDependencies(
        feed: ScriptedFeedRepository(
          loads: [
            [samplePost(id: 'initial')],
          ],
        ),
        engagement: FakeVideoCatalogRepository(forYouFeed: []),
        optional: FeedOptionalDependencies(updates: updates),
      ),
      updateRetry: FeedUpdateRetry(delays: const [Duration.zero]),
    );
    addTearDown(cubit.close);

    await cubit.load();
    await pumpEventQueue();

    expect(updates.watchCalls, 2);
  });
}
