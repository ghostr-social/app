part of 'warp_invalid_track_fallback_scenario.dart';

extension _WarpInvalidTrackFallbackAssertions
    on _WarpInvalidTrackFallbackDriver {
  void _expectLiveContract(
    _InvalidTrackFallbackEvidence evidence,
    _PlaybackAdvance advance,
  ) {
    _expectDefinitiveTransition(evidence);
    _expectReadinessIntegrity(evidence);
    _expectDecodedPlayback(evidence, advance);
    _expectNoNavigation();
    _expectOriginEvidence();
  }

  void _expectDefinitiveTransition(_InvalidTrackFallbackEvidence evidence) {
    final failures = scenario.failures.failures.where(
      (item) => _isDefinitiveFailure(item.failure),
    );
    expect(failures, hasLength(1));
    expect(evidence.failedStage.initializedAt, isNull);
    expect(evidence.failedStage.firstFrameAt, isNull);
    expect(evidence.failedStage.failedAt, isNotNull);
    expect(
      evidence.failedStage.failedAt,
      lessThanOrEqualTo(evidence.successfulStage.firstFrameAt!),
    );
    expect(
      evidence.failedStage.authority.representationId,
      isNot(evidence.successfulStage.authority.representationId),
    );
  }

  void _expectReadinessIntegrity(_InvalidTrackFallbackEvidence evidence) {
    final observations = graph.preparation.observations;
    final failedReady = observations.any(
      (item) => item.has(
        evidence.failedStage.authority,
        PlaybackPreparationReadiness.ready,
      ),
    );
    expect(failedReady, isFalse);
    expect(graph.preparation.observationsTruncated, isFalse);
    expect(
      graph.preparation.firstAt(
        evidence.successfulStage.authority,
        PlaybackPreparationReadiness.ready,
      ),
      isNotNull,
    );
  }

  void _expectDecodedPlayback(
    _InvalidTrackFallbackEvidence evidence,
    _PlaybackAdvance advance,
  ) {
    expect(evidence.successfulStage.initializedAt, isNotNull);
    expect(evidence.successfulStage.firstFrameAt, isNotNull);
    expect(graph.telemetry.probe.presentationFor(evidence.focus), isNotNull);
    expect(graph.telemetry.probe.playingLatency(evidence.focus), isNotNull);
    expect(advance.after, greaterThan(advance.before));
    expect(unavailableWasVisible, isFalse);
    expect(find.text('Video unavailable'), findsNothing);
  }

  void _expectNoNavigation() {
    final state = graph.cubit.state as FeedLoaded;
    expect(state.activeIndex, 0);
    expect(state.posts, hasLength(1));
    expect(graph.focus.occurrences.map((item) => item.videoId.value).toSet(), {
      scenario.events.single.id,
    });
    expect(graph.focus.hadTransportRescue, isFalse);
  }

  void _expectOriginEvidence() {
    final invalidPath = scenario.fixture.urlFor(origin).path;
    final invalid = _servedGets(invalidPath);
    final valid = _servedGets('/valid-rendition.mp4');
    expect(invalid, isNotEmpty);
    expect(valid, isNotEmpty);
    expect(invalid.fold(0, _addServedBytes), greaterThan(0));
    expect(valid.fold(0, _addServedBytes), greaterThan(0));
  }

  List<ProgressiveOriginRequest> _servedGets(String path) {
    return origin.requests
        .where((item) => item.method == 'GET' && item.path == path)
        .toList();
  }
}

int _addServedBytes(int total, ProgressiveOriginRequest request) {
  return total + request.servedBytes;
}
