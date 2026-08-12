import 'package:flutter/widgets.dart';

/// Makes vertical feed paging respond to short, deliberate swipes.
class FeedSwipePhysics extends ClampingScrollPhysics {
  const FeedSwipePhysics({required FeedSwipeGesture gesture, super.parent})
    : _gesture = gesture;

  static const double _dragMultiplier = 2.35;
  static const double _commitFraction = 0.2;
  static const double _maximumTravelFraction = 0.75;
  final FeedSwipeGesture _gesture;

  @override
  FeedSwipePhysics applyTo(ScrollPhysics? ancestor) {
    return FeedSwipePhysics(gesture: _gesture, parent: buildParent(ancestor));
  }

  @override
  double applyPhysicsToUserOffset(ScrollMetrics position, double offset) {
    final inheritedOffset = super.applyPhysicsToUserOffset(position, offset);
    if (!_gesture.tracking || position.outOfRange) return inheritedOffset;
    return _boundedGestureOffset(position, inheritedOffset);
  }

  double _boundedGestureOffset(ScrollMetrics position, double offset) {
    final viewport = position.viewportDimension;
    final origin = _gesture.originPixels ??= _pageOrigin(position, viewport);
    final travel = viewport * _maximumTravelFraction;
    final proposed = position.pixels - (offset * _dragMultiplier);
    final bounded = proposed
        .clamp(origin - travel, origin + travel)
        .clamp(position.minScrollExtent, position.maxScrollExtent)
        .toDouble();
    _gesture.targetPage = _gestureTarget(position, origin, bounded, viewport);
    return position.pixels - bounded;
  }

  double _pageOrigin(ScrollMetrics position, double viewport) {
    return (position.pixels / viewport).round() * viewport;
  }

  int? _gestureTarget(
    ScrollMetrics position,
    double origin,
    double proposed,
    double viewport,
  ) {
    final delta = proposed - origin;
    if (delta.abs() < viewport * _commitFraction) return null;
    final originPage = (origin / viewport).round();
    final target = originPage + delta.sign.toInt();
    final minimum = (position.minScrollExtent / viewport).round();
    final maximum = (position.maxScrollExtent / viewport).round();
    return target.clamp(minimum, maximum);
  }

  @override
  Simulation? createBallisticSimulation(
    ScrollMetrics position,
    double velocity,
  ) {
    if (position.outOfRange) {
      return super.createBallisticSimulation(position, velocity);
    }
    final target = _releaseTarget(position, velocity);
    if (_gesture.ended) _gesture.reset();
    if ((target - position.pixels).abs() < toleranceFor(position).distance) {
      return null;
    }
    return ScrollSpringSimulation(
      spring,
      position.pixels,
      target,
      velocity,
      tolerance: toleranceFor(position),
    );
  }

  double _releaseTarget(ScrollMetrics position, double velocity) {
    final viewport = position.viewportDimension;
    final origin = _gesture.originPixels;
    final originPage = ((origin ?? position.pixels) / viewport).round();
    final targetPage =
        _gesture.targetPage ??
        _flingTarget(originPage, velocity, position, viewport);
    return (targetPage * viewport)
        .clamp(position.minScrollExtent, position.maxScrollExtent)
        .toDouble();
  }

  int _flingTarget(
    int originPage,
    double velocity,
    ScrollMetrics position,
    double viewport,
  ) {
    if (velocity.abs() < minFlingVelocity) return originPage;
    final minimum = (position.minScrollExtent / viewport).round();
    final maximum = (position.maxScrollExtent / viewport).round();
    return (originPage + velocity.sign.toInt()).clamp(minimum, maximum);
  }

  @override
  double get minFlingDistance => 4;

  @override
  double get minFlingVelocity => 25;

  @override
  SpringDescription get spring => SpringDescription.withDampingRatio(
    mass: 0.2,
    stiffness: 1000,
    ratio: 1.05,
  );
}

final class FeedSwipeGesture {
  bool tracking = false;
  bool ended = false;
  double? originPixels;
  int? targetPage;

  void begin() {
    tracking = true;
    ended = false;
    originPixels = null;
    targetPage = null;
  }

  int? end() {
    tracking = false;
    ended = true;
    return targetPage;
  }

  void reset() {
    tracking = false;
    ended = false;
    originPixels = null;
    targetPage = null;
  }
}
