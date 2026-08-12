import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

import '../support/feed_swipe_metrics.dart';

void main() {
  test('a feed page already within tolerance does not animate', () {
    final gesture = FeedSwipeGesture()..begin();
    final physics = FeedSwipePhysics(gesture: gesture);
    final origin = feedSwipeMetrics();
    physics.applyPhysicsToUserOffset(origin, -0.0001);
    gesture.end();

    final simulation = physics.createBallisticSimulation(
      feedSwipeMetrics(pixels: 500.0001),
      0,
    );

    expect(simulation, isNull);
  });
}
