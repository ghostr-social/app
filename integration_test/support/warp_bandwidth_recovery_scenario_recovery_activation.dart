part of 'warp_bandwidth_recovery_scenario.dart';

typedef _RecoveryActivation = ({
  WarpSwipeBurst burst,
  ProgressiveOriginLinkProfile profile,
  int request,
});

Future<_RecoveryActivation> _activateRecovery(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _ImpairedFeed impaired,
  _RecoveryFocus recovery,
) async {
  final burst = await _swipeIntoRecovery(tester, journey, recovery);
  final profile = await journey.waitForBandwidthTrigger(
    tester,
    recovery.recoveryTrigger,
  );
  final request = recovery.recoveryTrigger.requestSequence!;
  expect(profile.activeRequestSequences, contains(request));
  _reportRecoveryActivation(recovery.recoveryTrigger, profile, request);
  await journey.waitForRequestProfiles(tester, request, {
    impaired.profile.generation,
    profile.generation,
  });
  return (burst: burst, profile: profile, request: request);
}

void _reportRecoveryActivation(
  ProgressiveOriginBandwidthTrigger trigger,
  ProgressiveOriginLinkProfile profile,
  int request,
) {
  debugPrint(
    'WARP_LINK recovery generation=${profile.generation} '
    'active=${profile.activeRequestSequences} request=$request '
    'path=${trigger.path} range=${trigger.requestRange} '
    'old_chunk=${trigger.confirmedEvent?.start}-'
    '${trigger.confirmedEvent?.end}',
  );
}
