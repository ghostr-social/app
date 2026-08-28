import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_roster.dart';
import 'package:ghostr/features/video_catalog/domain/use_cases/feed_session.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('an abandoned refresh preview cannot consume an accepted like', () {
    final original = samplePost();
    final accepted = original.withInteraction(
      VideoInteractionUpdate(
        likeCount: original.likeCount + 1,
        viewerHasLiked: true,
      ),
    );
    final session = FeedSession();
    session.loaded([original]);
    final visible = FeedRoster(session.liked([original], accepted));
    final confirming = accepted.withInteraction(
      VideoInteractionUpdate(
        likeCount: accepted.likeCount,
        viewerHasLiked: true,
        observations: const VideoMetricObservationUpdate(
          likes: VideoMetricObservation.observed,
        ),
      ),
    );
    final preview = session.captureResync(
      [confirming],
      eligible: [confirming],
      retainWatched: true,
    );
    session.previewResynced(visible, preview);
    final stale = original.withInteraction(
      VideoInteractionUpdate(
        likeCount: original.likeCount,
        viewerHasLiked: false,
        observations: const VideoMetricObservationUpdate(
          likes: VideoMetricObservation.observed,
        ),
      ),
    );

    final committed = session.resynced(
      visible,
      session.captureResync([stale], eligible: [stale], retainWatched: true),
    );

    expect(committed.active.viewerHasLiked, isTrue);
  });
}
