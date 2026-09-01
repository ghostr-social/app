part of 'warp_invalid_track_fallback_scenario.dart';

extension _WarpInvalidTrackFallbackWait on _WarpInvalidTrackFallbackDriver {
  Future<_InvalidTrackFallbackEvidence> _waitForFallback() async {
    _InvalidTrackFallbackEvidence? evidence;
    await _wait(() {
      evidence = _fallbackEvidence();
      return evidence != null;
    });
    return evidence!;
  }

  _InvalidTrackFallbackEvidence? _fallbackEvidence() {
    final failure = _definitiveFailure();
    final focus = graph.focus.publishedFor(scenario.events.single.id);
    if (failure == null || focus == null) return null;
    final stages = _fallbackStages(failure, focus);
    if (stages == null) return null;
    if (graph.telemetry.probe.playingLatency(focus) == null) return null;
    return _InvalidTrackFallbackEvidence(
      failure: failure,
      failedStage: stages.failed,
      successfulStage: stages.successful,
      focus: focus,
    );
  }

  _FallbackStages? _fallbackStages(
    WarpPlayerFailureEvidence failure,
    PlaybackFocus focus,
  ) {
    final presentation = graph.telemetry.probe.presentationFor(focus);
    final session = graph.telemetry.probe.sessionFor(focus);
    if (presentation == null || session == null) return null;
    final failedStage = _failedStage(failure);
    final successfulStage = graph.playerStages.forPresentation(
      session.deliveryId,
      presentation.elapsed,
    );
    if (!_isCompleteFallback(failure, failedStage, successfulStage)) {
      return null;
    }
    return (failed: failedStage!, successful: successfulStage!);
  }

  WarpPlayerFailureEvidence? _definitiveFailure() {
    return scenario.failures.failures
        .where((item) => _isDefinitiveFailure(item.failure))
        .firstOrNull;
  }

  WarpFeedPlayerStageEvidence? _failedStage(WarpPlayerFailureEvidence failure) {
    return graph.playerStages
        .attemptsFor(failure.authority.deliveryId)
        .where((stage) => stage.authority == failure.authority)
        .where((stage) => stage.failedAt != null)
        .firstOrNull;
  }

  bool _isCompleteFallback(
    WarpPlayerFailureEvidence failure,
    WarpFeedPlayerStageEvidence? failed,
    WarpFeedPlayerStageEvidence? successful,
  ) {
    return failed != null &&
        successful?.firstFrameAt != null &&
        successful!.authority != failure.authority;
  }

  Future<_PlaybackAdvance> _waitForAdvancement(PlaybackFocus focus) async {
    final probe = graph.telemetry.probe;
    final before = probe.latestPositionFor(focus) ?? Duration.zero;
    await _pumpFor(const Duration(seconds: 1));
    await _wait(
      () => (probe.latestPositionFor(focus) ?? Duration.zero) > before,
      timeout: const Duration(seconds: 5),
    );
    return (before: before, after: probe.latestPositionFor(focus)!);
  }
}
