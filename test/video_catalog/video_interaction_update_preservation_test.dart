import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/sample_data.dart';

void main() {
  test('a partial interaction update preserves omitted comment state', () {
    final observed = samplePost().withInteraction(
      const VideoInteractionUpdate(
        likeCount: 42,
        viewerHasLiked: false,
        commentCount: 9,
        observations: VideoMetricObservationUpdate(
          likes: VideoMetricObservation.observed,
          comments: VideoMetricObservation.observed,
        ),
      ),
    );

    final updated = observed.withInteraction(
      const VideoInteractionUpdate(
        likeCount: 43,
        viewerHasLiked: true,
        observations: VideoMetricObservationUpdate(
          likes: VideoMetricObservation.unobserved,
        ),
      ),
    );

    expect(updated.likeCount, 43);
    expect(updated.viewerHasLiked, isTrue);
    expect(updated.commentCount, 9);
    expect(updated.metrics.likeObservation, VideoMetricObservation.unobserved);
    expect(updated.metrics.commentObservation, VideoMetricObservation.observed);
  });
}
