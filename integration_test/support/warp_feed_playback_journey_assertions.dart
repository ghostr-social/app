part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyAssertions on WarpFeedPlaybackJourney {
  Future<void> verifyReadyBurstPlayback(
    WarpFeedPreparationObservation snapshot,
    List<PlaybackFocus> focuses,
    List<Duration> releases,
  ) async {
    if (focuses.length != releases.length) {
      throw ArgumentError('Each Ready focus requires its gesture release.');
    }
    final violations = <PlaybackFocus>[];
    for (var index = 0; index < focuses.length; index += 1) {
      final requirePlaying = deviceReadyBurstRequiresPlaying(
        index,
        focuses.length,
      );
      if (_violates(
        snapshot,
        focuses[index],
        releases[index],
        requirePlaying,
      )) {
        violations.add(focuses[index]);
      }
    }
    if (violations.isNotEmpty) {
      await reportSchedulingEvidence();
      for (final focus in violations) {
        debugPrint('WARP_PLAYBACK_VIOLATION ${_playbackEvidence(focus)}');
      }
    }
    for (var index = 0; index < focuses.length; index += 1) {
      _expectReadyPlayback(
        snapshot,
        focuses[index],
        releases[index],
        requirePlaying: deviceReadyBurstRequiresPlaying(index, focuses.length),
      );
    }
  }

  Future<void> verifyReadyPlayback(
    WarpFeedPreparationObservation snapshot,
    PlaybackFocus focus,
    Duration releasedAt,
  ) async {
    if (_violates(snapshot, focus, releasedAt, true)) {
      await reportSchedulingEvidence();
      debugPrint('WARP_PLAYBACK_VIOLATION ${_playbackEvidence(focus)}');
    }
    _expectReadyPlayback(snapshot, focus, releasedAt, requirePlaying: true);
  }

  void verifyReversePlayback(
    List<PlaybackFocus> focuses,
    List<Duration> releases,
  ) {
    if (focuses.length != releases.length || focuses.isEmpty) {
      throw ArgumentError('Each reverse focus requires a gesture release.');
    }
    for (var index = 0; index < focuses.length; index += 1) {
      final focus = focuses[index];
      final evidence = _playbackEvidence(focus);
      final presented = telemetry.probe.firstFrameLatency(focus);
      expect(presented, isNotNull, reason: evidence);
      expect(
        _transitionLatency(focus, presented!, releases[index]),
        lessThanOrEqualTo(deviceProtectedTransitionTarget),
        reason: evidence,
      );
      _expectHealthyPlayback(focus, evidence);
    }
    final finalFocus = focuses.last;
    final playing = telemetry.probe.playingLatency(finalFocus);
    final evidence = _playbackEvidence(finalFocus);
    expect(playing, isNotNull, reason: evidence);
    expect(
      _transitionLatency(finalFocus, playing!, releases.last),
      lessThanOrEqualTo(deviceProtectedTransitionTarget),
      reason: evidence,
    );
  }
}
