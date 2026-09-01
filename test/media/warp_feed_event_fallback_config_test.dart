import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/warp_feed_event_config.dart';

void main() {
  test('next event keeps its primary and advertises the rescue fallback', () {
    const config = SignedWarpFeedConfig(
      eventCount: 4,
      candidateLayout: WarpFeedCandidateLayout.nextWithRescue,
    );

    final current = config.sourcesFor('current');
    final next = config.sourcesFor('next');

    expect(current.primaryLabel, 'current');
    expect(current.fallbackLabel, isNull);
    expect(next.primaryLabel, 'next');
    expect(next.fallbackLabel, 'next-rescue');
  });
}
