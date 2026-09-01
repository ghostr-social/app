part of 'warp_invalid_track_fallback_scenario.dart';

extension _WarpInvalidTrackFallbackBounded on _WarpInvalidTrackFallbackDriver {
  void _expectQuiescent(_InvalidTrackFallbackEvidence evidence) {
    final stages = graph.playerStages.attemptsFor(
      evidence.failure.authority.deliveryId,
    );
    expect(
      stages.map((item) => item.authority.representationId).toSet(),
      hasLength(2),
    );
    expect(stages.length, lessThanOrEqualTo(4));
    expect(stages.every((item) => item.isTerminal), isTrue);
    expect(peakMountedPlayers, lessThanOrEqualTo(2));
    expect(peakControllerCapacity, lessThanOrEqualTo(8));
    expect(
      videoPlaybackCapacityOf(scenario.playback),
      emptyVideoPlaybackCapacitySnapshot,
    );
    expect(origin.maximumConcurrentResponses, lessThanOrEqualTo(4));
    expect(origin.requests.length, lessThanOrEqualTo(24));
    expect(origin.activeIncompleteRequestSequences, isEmpty);
    expect(find.byType(VideoPlayer, skipOffstage: false), findsNothing);
  }

  void _report(
    _InvalidTrackFallbackEvidence evidence,
    _PlaybackAdvance advance,
  ) {
    debugPrint(
      'WARP_INVALID_TRACK_FALLBACK failure=${evidence.failure.failure.name} '
      'failed_rep=${evidence.failedStage.authority.representationId.value} '
      'selected_rep=${evidence.successfulStage.authority.representationId.value} '
      'attempts=${graph.playerStages.progressiveAttemptCount} '
      'requests=${origin.requests.length} players=$peakMountedPlayers '
      'controller_peak=$peakControllerCapacity '
      'advance_ms=${(advance.after - advance.before).inMilliseconds}',
    );
  }
}
