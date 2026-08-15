import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_viewer.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_tracker.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  test('a visible revision records every new media identity', () async {
    final history = FakeWatchHistoryRepository();
    final viewer = FeedViewer(
      watchTracker: WatchHistoryTracker(
        history: history,
        failureReporter: RecordingFailureReporter(),
      ),
    );
    const revisionUrl = 'https://cdn.example/revision.mp4';
    final first = samplePost(id: 'coordinate');
    final revision = first.withMedia(VideoMediaSource.remote(revisionUrl));
    final republished = samplePost(
      id: 'new-event',
    ).withMedia(VideoMediaSource.remote(revisionUrl));

    viewer.landedOn([first], 0);
    await pumpEventQueue();
    viewer.rosterChanged([revision], 0);
    await pumpEventQueue();

    expect(await history.filterUnwatched([republished]), isEmpty);
  });
}
