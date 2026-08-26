import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../../integration_test/support/warp_feed_preparation_probe.dart';

void main() {
  test('preparation snapshot history is bounded and reports truncation', () {
    final metrics = WarpFeedPreparationMetrics(() => Duration.zero);
    for (var revision = 1; revision <= 300; revision += 1) {
      metrics.observe(
        PlaybackPreparationPlan(
          revision: BigInt.from(revision),
          currentDeliveryId: null,
        ),
      );
    }

    expect(metrics.observations, hasLength(256));
    expect(metrics.observationsTruncated, isTrue);
    expect(metrics.observations.first.revision, BigInt.from(45));
    expect(metrics.latest.revision, BigInt.from(300));
  });
}
