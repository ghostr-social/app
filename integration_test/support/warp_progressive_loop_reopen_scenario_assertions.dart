part of 'warp_progressive_loop_reopen_scenario.dart';

void _expectPromotionAndLoop(
  WarpProgressivePromotionEvidence promotion,
  WarpProgressiveLoopEvidence loop,
) {
  expect(promotion.rangedResponses, greaterThan(1));
  expect(promotion.uniqueBytes, promotion.totalBytes);
  expect(promotion.duplicateBytes, 0);
  expect(loop.beforeReset, greaterThanOrEqualTo(const Duration(seconds: 5)));
  expect(loop.reset, lessThanOrEqualTo(const Duration(seconds: 1)));
  expect(loop.afterReset, greaterThan(loop.reset));
}

void _expectOriginalPlaybackIdentity(_OpenedLoopFeed opened) {
  final probe = opened.journey.telemetry.probe;
  expect(probe.sessionFor(opened.focus), opened.session);
  final activations = probe.activations
      .where((candidate) => candidate.videoId == opened.session.videoId)
      .toList();
  expect(activations, hasLength(1));
  expect(activations.single, opened.session);
  final attempts = opened.journey.playerStages.attemptsFor(
    opened.session.deliveryId,
  );
  expect(attempts, hasLength(1));
  expect(attempts.single, same(opened.attempt));
  expect(opened.attempt.failedAt, isNull);
  expect(opened.attempt.releasedAt, isNull);
}

void _expectHealthyPlayback(_OpenedLoopFeed opened) {
  final journey = opened.journey;
  expect(
    journey.telemetry.probe.hasPhaseFor(opened.focus, PlaybackPhase.failed),
    isFalse,
  );
  expect(
    journey.telemetry.probe.hasPhaseFor(
      opened.focus,
      PlaybackPhase.networkStalled,
    ),
    isFalse,
  );
  expect(journey.hadPlaybackError, isFalse);
  expect(find.text('Video unavailable'), findsNothing);
}

void _reportLoop(
  WarpProgressivePromotionEvidence promotion,
  WarpProgressiveLoopEvidence loop,
) {
  debugPrint(
    'WARP_LOOP ranged=${promotion.rangedResponses} '
    'coverage=${promotion.uniqueBytes}/${promotion.totalBytes} '
    'duplicate=${promotion.duplicateBytes} '
    'positions_ms=${loop.beforeReset.inMilliseconds}/'
    '${loop.reset.inMilliseconds}/${loop.afterReset.inMilliseconds}',
  );
}
