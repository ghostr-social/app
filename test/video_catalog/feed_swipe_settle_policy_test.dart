import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_swipe_physics.dart';

void main() {
  test('feed page settling uses a fast damped spring', () {
    final spring = FeedSwipePhysics(gesture: FeedSwipeGesture()).spring;

    expect(spring.mass, 0.2);
    expect(spring.stiffness, 1000);
    expect(spring.damping, closeTo(29.7, 0.1));
  });
}
