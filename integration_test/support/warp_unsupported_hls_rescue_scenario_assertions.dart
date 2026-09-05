part of 'warp_unsupported_hls_rescue_scenario.dart';

extension _WarpUnsupportedHlsRescueAssertions
    on _WarpUnsupportedHlsRescueDriver {
  void _expectLiveContract(_UnsupportedHlsEvidence evidence) {
    _expectTypedHlsRejection(evidence);
    _expectManualNavigation(evidence);
    _expectDecodedAlternate(evidence);
    _expectBoundedOrigins(this);
  }

  void _expectTypedHlsRejection(_UnsupportedHlsEvidence evidence) {
    expect(
      evidence.failure.detail,
      'HLS bootstrap was blocked by media policy',
    );
    expect(evidence.failure.hlsAuthority, isNull);
    expect(_failedSnapshots(), isNotEmpty);
    expect(
      _failedSnapshots().any(
        (snapshot) => snapshot.phase == VideoDeliveryPhase.startable,
      ),
      isFalse,
    );
    expect(
      graph.telemetry.probe.firstFrameLatency(evidence.failedFocus),
      isNull,
    );
    expect(graph.telemetry.probe.playingLatency(evidence.failedFocus), isNull);
    final leases = runtime.hlsGateway.acquisitions;
    expect(leases, hasLength(1));
    expect(leases.single.deliveryId, failedDeliveryId);
    expect(leases.single.expectedAuthority, isNull);
    expect(leases.single.authority, isNull);
  }

  void _expectManualNavigation(_UnsupportedHlsEvidence evidence) {
    expect(evidence.alternateFocus.cause, FeedFocusCause.userNavigation);
    expect(evidence.alternateFocus.rescue, isNull);
    expect(graph.focus.hadTransportRescue, isFalse);
    expect(feed.activeIndex, 1);
    expect(
      graph.focus.occurrencesFor(runtime.events[1].id),
      everyElement(
        isA<PlaybackFocus>().having(
          (focus) => focus.cause,
          'cause',
          FeedFocusCause.userNavigation,
        ),
      ),
    );
  }

  void _expectDecodedAlternate(_UnsupportedHlsEvidence evidence) {
    final stages = graph.playerStages.attemptsFor(alternateDeliveryId);
    expect(stages, isNotEmpty);
    expect(stages.any((stage) => stage.firstFrameAt != null), isTrue);
    expect(
      graph.telemetry.probe.presentationFor(evidence.alternateFocus),
      isNotNull,
    );
    expect(
      graph.telemetry.probe.playingLatency(evidence.alternateFocus),
      isNotNull,
    );
    expect(evidence.after, greaterThan(evidence.before));
    expect(_alternateWasPlayerReady(), isTrue);
    expect(unavailableWasVisible, isTrue);
    expect(find.text('Video unavailable'), findsNothing);
  }

  List<VideoDeliverySnapshot> _failedSnapshots() => graph
      .deliveryProbe
      .observations
      .map((item) => item.snapshot)
      .where((snapshot) => snapshot.deliveryId == failedDeliveryId)
      .toList();

  bool _alternateWasPlayerReady() => graph.preparation.observations.any(
    (plan) => plan.upcoming.any(
      (asset) =>
          asset.authority.deliveryId == alternateDeliveryId &&
          asset.readiness == PlaybackPreparationReadiness.ready,
    ),
  );
}
