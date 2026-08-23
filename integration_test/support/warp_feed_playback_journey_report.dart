part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyReport on WarpFeedPlaybackJourney {
  void reportStartup(PlaybackFocus focus) {
    final firstFrame = telemetry.probe.firstFrameLatency(focus)?.inMilliseconds;
    final progress = telemetry.probe.playingLatency(focus)?.inMilliseconds;
    debugPrint(
      'WARP_QOE startup_ms=$firstFrame progress_ms=$progress '
      '${_stageEvidence(focus)} origin=${_originEvidence()}',
    );
  }

  void reportFinal(PlaybackFocus focus) {
    final firstFrame = telemetry.probe.firstFrameLatency(focus)?.inMilliseconds;
    final progress = telemetry.probe.playingLatency(focus)?.inMilliseconds;
    debugPrint(
      'WARP_QOE focus_switch_ms=$firstFrame progress_ms=$progress '
      '${_stageEvidence(focus)}',
    );
  }

  String _stageEvidence(PlaybackFocus focus) {
    final presentation = telemetry.probe.presentationFor(focus);
    if (presentation == null) return 'stages=unavailable';
    final deliveryId = presentation.session.deliveryId;
    final player = playerStages.latestFor(
      deliveryId,
      noLaterThan: presentation.elapsed,
    );
    final structural = preparation.firstCurrentAt(
      deliveryId,
      PlaybackPreparationReadiness.structuralStartable,
    );
    return 'rust_structural_startable_ms=${_deltaMs(structural, focus)} '
        'player_prepare_ms=${_deltaMs(player?.preparedAt, focus)} '
        'initialize_start_ms=${_deltaMs(player?.initializingAt, focus)} '
        'initialized_ms=${_deltaMs(player?.initializedAt, focus)} '
        'native_frame_ms=${_deltaMs(player?.firstFrameAt, focus)} '
        'presented_ms=${_deltaMs(presentation.elapsed, focus)}';
  }

  String _originEvidence() {
    return resources.origin.requests
        .map((request) {
          final range = request.range;
          final span = range == null ? 'full' : '${range.start}-${range.end}';
          return '${request.method}:${request.path}:$span';
        })
        .join(',');
  }
}

String _deltaMs(Duration? observed, PlaybackFocus focus) {
  return observed == null
      ? 'na'
      : '${(observed - focus.startedAt).inMilliseconds}';
}
