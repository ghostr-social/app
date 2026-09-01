part of 'warp_unsupported_hls_rescue_scenario.dart';

extension _WarpUnsupportedHlsRescueAssertions
    on _WarpUnsupportedHlsRescueDriver {
  void _expectLiveContract(_UnsupportedHlsEvidence evidence) {
    _expectTypedHlsRejection(evidence);
    _expectAutomaticRescue(evidence);
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

  void _expectAutomaticRescue(_UnsupportedHlsEvidence evidence) {
    expect(evidence.rescueFocus.cause, FeedFocusCause.transportRescue);
    expect(
      evidence.rescueFocus.rescue?.reason,
      FeedTransportRescueReason.deliveryFailed,
    );
    expect(evidence.rescueFocus.rescue?.rankDisplacement, 1);
    expect(evidence.rescueFocus.rescue?.wait, Duration.zero);
    expect(feed.activeIndex, 1);
    expect(
      graph.focus.occurrencesFor(runtime.events[1].id),
      everyElement(
        isA<PlaybackFocus>().having(
          (focus) => focus.cause,
          'cause',
          FeedFocusCause.transportRescue,
        ),
      ),
    );
  }

  void _expectDecodedAlternate(_UnsupportedHlsEvidence evidence) {
    final stages = graph.playerStages.attemptsFor(alternateDeliveryId);
    expect(stages, isNotEmpty);
    expect(stages.any((stage) => stage.firstFrameAt != null), isTrue);
    expect(
      graph.telemetry.probe.presentationFor(evidence.rescueFocus),
      isNotNull,
    );
    expect(
      graph.telemetry.probe.playingLatency(evidence.rescueFocus),
      isNotNull,
    );
    expect(evidence.after, greaterThan(evidence.before));
    expect(_alternateWasPlayerReady(), isTrue);
    expect(unavailableWasVisible, isFalse);
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
