import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../../integration_test/support/warp_feed_preparation_probe.dart';

void main() {
  test('preparation snapshots reject lower and retain equal revisions', () {
    var elapsed = const Duration(milliseconds: 10);
    final metrics = WarpFeedPreparationMetrics(() => elapsed);
    metrics.observe(_emptyPlan(2));
    elapsed = const Duration(milliseconds: 20);
    metrics.observe(_emptyPlan(1));

    expect(metrics.observations, hasLength(1));
    expect(metrics.latest.revision, BigInt.two);
    expect(metrics.latest.elapsed, const Duration(milliseconds: 10));

    elapsed = const Duration(milliseconds: 30);
    metrics.observe(_emptyPlan(2));

    expect(metrics.observations, hasLength(2));
    expect(
      metrics.observations.first.elapsed,
      const Duration(milliseconds: 10),
    );
    expect(metrics.latest.revision, BigInt.two);
    expect(metrics.latest.elapsed, elapsed);
  });
}

PlaybackPreparationPlan _emptyPlan(int revision) => PlaybackPreparationPlan(
  revision: BigInt.from(revision),
  currentDeliveryId: null,
);
