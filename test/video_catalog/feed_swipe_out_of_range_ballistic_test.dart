import 'package:flutter/physics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

import '../support/feed_swipe_metrics.dart';

void main() {
  test('out-of-range feed motion delegates to clamping ballistics', () {
    final physics = FeedSwipePhysics(gesture: FeedSwipeGesture());
    final metrics = feedSwipeMetrics(pixels: -20);

    final simulation = physics.createBallisticSimulation(metrics, 0);

    expect(simulation, isA<ScrollSpringSimulation>());
    expect(simulation!.x(10), closeTo(metrics.minScrollExtent, 0.01));
  });
}
