part of 'warp_bandwidth_recovery_scenario.dart';

typedef _RecoveredEvidence = ({
  ProgressiveOriginLinkProfile profile,
  ProgressiveOriginLinkWindow window,
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
  PlaybackFocus focus,
  String frontierPath,
});

Future<_RecoveredEvidence> _measureRecovery(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
  _RecoveryFocus recovery,
) async {
  final journey = opened.journey;
  final active = await _activateRecovery(tester, journey, impaired, recovery);
  final window = await journey.waitForConfirmedLinkWindow(
    tester,
    active.profile.generation,
    minimumDuration: const Duration(milliseconds: 500),
  );
  _expectRecoveryWindow(active.request, active.profile, window);
  final focus = active.burst.focuses.last;
  final confirmedAtEpochMs = _recoveryConfirmationFence(recovery);
  final paired = await _waitForRecoveryPair(tester, journey, (
    recovery: recovery,
    focus: focus,
    confirmedAtEpochMs: confirmedAtEpochMs,
  ));
  _reportNetworkResponse('recovery', paired.decision, confirmedAtEpochMs);
  return (
    profile: active.profile,
    window: window,
    decision: paired.decision,
    plan: paired.plan,
    focus: focus,
    frontierPath: recovery.frontier.firstUnreadyPath,
  );
}

int _recoveryConfirmationFence(_RecoveryFocus recovery) {
  return recovery.recoveryTrigger.confirmedEvent!.confirmedAtEpochMs!;
}

void _expectRecoveryWindow(
  int request,
  ProgressiveOriginLinkProfile profile,
  ProgressiveOriginLinkWindow window,
) {
  expect(window.events, isNotEmpty);
  expect(
    window.events.any((event) => event.requestSequence == request),
    isTrue,
  );
  expect(window.duration, greaterThan(Duration.zero));
  expect(window.achievedBandwidthKbps, lessThanOrEqualTo(2500));
  expect(profile.activeRequestSequences, contains(request));
}
