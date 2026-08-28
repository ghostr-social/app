part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyPlaybackContract on WarpFeedPlaybackJourney {
  bool _violates(
    WarpFeedPreparationObservation snapshot,
    PlaybackFocus focus,
    Duration releasedAt,
    bool requirePlaying,
  ) {
    final presented = telemetry.probe.firstFrameLatency(focus);
    final playing = telemetry.probe.playingLatency(focus);
    return !isReadyIn(snapshot, focus) ||
        presented == null ||
        _transitionLatency(focus, presented, releasedAt) >
            deviceProtectedTransitionTarget ||
        (requirePlaying &&
            (playing == null ||
                _transitionLatency(focus, playing, releasedAt) >
                    deviceProtectedTransitionTarget)) ||
        telemetry.probe.hasPhaseFor(focus, PlaybackPhase.failed) ||
        telemetry.probe.hasPhaseFor(focus, PlaybackPhase.networkStalled);
  }

  void _expectReadyPlayback(
    WarpFeedPreparationObservation snapshot,
    PlaybackFocus focus,
    Duration releasedAt, {
    required bool requirePlaying,
  }) {
    final evidence = _playbackEvidence(focus);
    expect(isReadyIn(snapshot, focus), isTrue, reason: evidence);
    expect(
      telemetry.probe.firstFrameLatency(focus),
      isNotNull,
      reason: evidence,
    );
    final presented = telemetry.probe.firstFrameLatency(focus)!;
    expect(
      _transitionLatency(focus, presented, releasedAt),
      lessThanOrEqualTo(deviceProtectedTransitionTarget),
      reason: evidence,
    );
    _expectHealthyPlayback(focus, evidence);
    if (!requirePlaying) return;
    final playing = telemetry.probe.playingLatency(focus);
    expect(playing, isNotNull, reason: evidence);
    final transition = _transitionLatency(focus, playing!, releasedAt);
    expect(
      transition,
      lessThanOrEqualTo(deviceProtectedTransitionTarget),
      reason: evidence,
    );
  }

  void _expectHealthyPlayback(PlaybackFocus focus, String evidence) {
    expect(
      telemetry.probe.hasPhaseFor(focus, PlaybackPhase.failed),
      isFalse,
      reason: evidence,
    );
    expect(
      telemetry.probe.hasPhaseFor(focus, PlaybackPhase.networkStalled),
      isFalse,
      reason: evidence,
    );
  }

  Duration _transitionLatency(
    PlaybackFocus focus,
    Duration evidenceLatency,
    Duration releasedAt,
  ) {
    return focus.startedAt - releasedAt + evidenceLatency;
  }
}
