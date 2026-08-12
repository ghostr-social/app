import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

import '../support/feed_swipe_metrics.dart';

void main() {
  test('an uncommitted feed fling targets one adjacent page', () {
    final gesture = FeedSwipeGesture()..begin();
    final physics = FeedSwipePhysics(gesture: gesture);
    final origin = feedSwipeMetrics();
    final offset = physics.applyPhysicsToUserOffset(origin, -20);
    gesture.end();

    final simulation = physics.createBallisticSimulation(
      feedSwipeMetrics(pixels: origin.pixels - offset),
      500,
    );

    expect(simulation, isNotNull);
    expect(simulation!.x(10), closeTo(1000, 0.01));
  });
}
