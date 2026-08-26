part of 'warp_feed_preparation_probe.dart';

final class WarpFeedPreparationMetrics {
  WarpFeedPreparationMetrics(this._clock, [this._externalSequence]);

  static const _retentionLimit = 256;

  final WarpFeedPreparationClock _clock;
  final WarpFeedPreparationSequence? _externalSequence;
  final _observations = <WarpFeedPreparationObservation>[];
  var _localSequence = 0;
  var maximumStructuralDepth = 0;
  var maximumReadyDepth = 0;
  var observationsTruncated = false;

  List<WarpFeedPreparationObservation> get observations =>
      List.unmodifiable(_observations);

  WarpFeedPreparationObservation get latest => _observations.last;

  void observe(PlaybackPreparationPlan plan) {
    if (_isOlder(plan.revision)) return;
    final observation = WarpFeedPreparationObservation.fromPlan(
      plan,
      _clock(),
      _markSequence(),
    );
    _observations.add(observation);
    _updateMaximumDepths(observation);
    _boundObservations();
  }

  WarpFeedPreparationObservation? atOrBefore(Duration elapsed) {
    for (final observation in _observations.reversed) {
      if (observation.elapsed <= elapsed) return observation;
    }
    return null;
  }

  WarpFeedPreparationObservation? atOrBeforeSequence(int sequence) {
    for (final observation in _observations.reversed) {
      if (observation.sequence <= sequence) return observation;
    }
    return null;
  }

  Duration? firstAt(
    PlaybackAssetAuthority authority,
    PlaybackPreparationReadiness readiness,
  ) {
    for (final snapshot in _observations) {
      if (snapshot.has(authority, readiness)) return snapshot.elapsed;
    }
    return null;
  }

  Duration? firstStructurallyStartableAt(PlaybackAssetAuthority authority) {
    for (final snapshot in _observations) {
      if (snapshot.hasStructurallyStartable(authority)) return snapshot.elapsed;
    }
    return null;
  }

  bool _isOlder(BigInt revision) {
    return _observations.isNotEmpty && revision < latest.revision;
  }

  int _markSequence() => _externalSequence?.call() ?? ++_localSequence;

  void _updateMaximumDepths(WarpFeedPreparationObservation observation) {
    if (observation.structuralDepth > maximumStructuralDepth) {
      maximumStructuralDepth = observation.structuralDepth;
    }
    if (observation.readyDepth > maximumReadyDepth) {
      maximumReadyDepth = observation.readyDepth;
    }
  }

  void _boundObservations() {
    if (_observations.length <= _retentionLimit) return;
    _observations.removeAt(0);
    observationsTruncated = true;
  }
}
