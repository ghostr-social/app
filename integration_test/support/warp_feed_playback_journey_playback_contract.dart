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
        (requirePlaying && playing == null) ||
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
    _expectHealthyPlayback(focus, evidence);
    if (!requirePlaying) return;
    expect(telemetry.probe.playingLatency(focus), isNotNull, reason: evidence);
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
}
