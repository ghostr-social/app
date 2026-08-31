part of 'warp_bandwidth_recovery_scenario.dart';

Future<_ImpairedFeed> _impairSharedLink(
  WidgetTester tester,
  _PacedFeed opened,
) async {
  final journey = opened.journey;
  final profile = await journey.waitForBandwidthTrigger(
    tester,
    opened.lossTrigger,
  );
  final request = opened.lossTrigger.requestSequence!;
  expect(profile.activeRequestSequences, contains(request));
  debugPrint(
    'WARP_LINK loss generation=${profile.generation} '
    'active=${profile.activeRequestSequences} request=$request '
    'path=${opened.lossTrigger.path} '
    'range=${opened.lossTrigger.requestRange} '
    'old_chunk=${opened.lossTrigger.confirmedEvent?.start}-'
    '${opened.lossTrigger.confirmedEvent?.end}',
  );
  await journey.waitForRequestProfiles(tester, request, {
    opened.fastProfile.generation,
    profile.generation,
  });
  final window = await journey.waitForConfirmedLinkWindow(
    tester,
    profile.generation,
    minimumDuration: const Duration(milliseconds: 1500),
  );
  _expectImpairmentWindow(window);
  final decision = await _waitForLossDecision(
    tester,
    opened,
    window.confirmedAtEpochMs,
  );
  _reportNetworkResponse('loss', decision, window.confirmedAtEpochMs);
  final ready = await _waitForImpairedReady(tester, opened);
  return (profile: profile, window: window, decision: decision, ready: ready);
}

void _expectImpairmentWindow(ProgressiveOriginLinkWindow window) {
  expect(window.events, isNotEmpty);
  expect(
    window.duration,
    greaterThanOrEqualTo(const Duration(milliseconds: 1500)),
  );
  expect(window.achievedBandwidthKbps, lessThanOrEqualTo(700));
}

Future<WarpReadyWindow> _waitForImpairedReady(
  WidgetTester tester,
  _PacedFeed opened,
) {
  final journey = opened.journey;
  final deliveryId = journey.telemetry.probe
      .sessionFor(opened.startup)!
      .deliveryId;
  return journey.waitForReadyBurstWindow(
    tester,
    opened.focusGeneration,
    currentDeliveryId: deliveryId,
    minimumDepth: 2,
    afterSequence: opened.startup.sequence,
  );
}
