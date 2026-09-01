part of 'warp_invalid_track_fallback_scenario.dart';

extension _WarpInvalidTrackFallbackWaitCore on _WarpInvalidTrackFallbackDriver {
  Future<void> _wait(
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick();
    }
    if (!condition()) fail(_timeoutEvidence(timeout));
  }

  Future<void> _pumpFor(Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tick();
    }
  }

  Future<void> _tick() async {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    _sampleVisibleState();
  }

  void _sampleVisibleState() {
    final mounted = find
        .byType(VideoPlayer, skipOffstage: false)
        .evaluate()
        .length;
    if (mounted > peakMountedPlayers) peakMountedPlayers = mounted;
    final capacity = videoPlaybackCapacityOf(scenario.playback).inUse;
    if (capacity > peakControllerCapacity) peakControllerCapacity = capacity;
    unavailableWasVisible |= find
        .text('Video unavailable')
        .evaluate()
        .isNotEmpty;
  }

  String _timeoutEvidence(Duration timeout) {
    return 'Invalid-track fallback timed out after $timeout; '
        'state=${graph.cubit.state.runtimeType}, '
        'failures=${scenario.failures.failures.map((item) => item.failure)}, '
        'attempts=${graph.playerStages.progressiveAttemptCount}, '
        'capacity=${videoPlaybackCapacityOf(scenario.playback)}, '
        'requests=${origin.requests.length}, '
        'active=${origin.activeIncompleteRequestSequences}.';
  }
}

bool _isDefinitiveFailure(PlayerPreparationFailureKind failure) {
  return failure == PlayerPreparationFailureKind.invalidVideoTrack ||
      failure == PlayerPreparationFailureKind.decoderUnsupported;
}
