part of 'warp_unsupported_hls_rescue_scenario.dart';

void _expectBoundedOrigins(_WarpUnsupportedHlsRescueDriver driver) {
  final progressive = driver.progressive;
  final hls = progressive.encryptedHlsRequests;
  final encrypted = progressive.requests
      .where((request) => request.path.startsWith('/encrypted/'))
      .toList();
  expect(hls.length, inInclusiveRange(1, 2));
  expect(encrypted, hls);
  expect(hls.every((request) => request.method == 'GET'), isTrue);
  expect(
    hls.every((request) => request.path == '/encrypted/index.m3u8'),
    isTrue,
  );
  expect(
    progressive.encryptedHlsUrl.origin,
    progressive.urlFor('unsupported-hls-rescue').origin,
  );
  expect(progressive.requestsFor('unsupported-hls-rescue'), isNotEmpty);
  expect(progressive.requests.length, lessThanOrEqualTo(24));
  expect(progressive.maximumConcurrentResponses, lessThanOrEqualTo(1));
}

extension _WarpUnsupportedHlsRescueBounded on _WarpUnsupportedHlsRescueDriver {
  Future<void> _waitForQuiescence() {
    return _wait(_isQuiescent, timeout: const Duration(seconds: 15));
  }

  bool _isQuiescent() {
    final stages = graph.playerStages.attemptsFor(alternateDeliveryId);
    return stages.isNotEmpty &&
        stages.every((stage) => stage.isTerminal) &&
        videoPlaybackCapacityOf(graph.playback).isQuiescent &&
        progressive.activeIncompleteRequestSequences.isEmpty &&
        find.byType(VideoPlayer, skipOffstage: false).evaluate().isEmpty;
  }

  void _expectBoundedCleanup(_UnsupportedHlsEvidence evidence) {
    expect(graph.playerStages.progressiveAttemptCount, lessThanOrEqualTo(2));
    expect(
      graph.playerStages
          .attemptsFor(alternateDeliveryId)
          .every((stage) => stage.isTerminal),
      isTrue,
    );
    expect(peakMountedPlayers, lessThanOrEqualTo(2));
    expect(peakControllerCapacity, lessThanOrEqualTo(2));
    expect(
      videoPlaybackCapacityOf(graph.playback),
      emptyVideoPlaybackCapacitySnapshot,
    );
    expect(progressive.activeIncompleteRequestSequences, isEmpty);
    expect(runtime.hlsGateway.acquisitions, hasLength(1));
    expect(runtime.hlsGateway.acquisitions.single.released, isTrue);
    expect(runtime.hlsGateway.activeFor(failedDeliveryId), isEmpty);
    expect(find.byType(VideoPlayer, skipOffstage: false), findsNothing);
    expect(evidence.failure.phase, VideoDeliveryPhase.failed);
  }

  void _report(_UnsupportedHlsEvidence evidence) {
    debugPrint(
      'WARP_UNSUPPORTED_HLS failure=policy '
      'hls_requests=${progressive.encryptedHlsRequests.length} '
      'gateway_acquisitions=${runtime.hlsGateway.acquisitions.length} '
      'navigation=${evidence.alternateFocus.cause.name} rescues=0 '
      'progressive_requests=${progressive.requests.length} '
      'player_attempts=${graph.playerStages.progressiveAttemptCount} '
      'player_peak=$peakMountedPlayers controller_peak=$peakControllerCapacity '
      'advance_ms=${(evidence.after - evidence.before).inMilliseconds}',
    );
  }
}
