import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:integration_test/integration_test.dart';

import 'support/device_hls_reactivation_journey.dart';
import 'support/device_playback_testbed.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'inactive HLS revokes its decoded authority before reactivation',
    _runHlsReactivationContract,
  );
}

Future<void> _runHlsReactivationContract(WidgetTester tester) async {
  final journey = await DeviceHlsReactivationJourney.start();
  addTearDown(journey.close);
  final first = await _present(tester, journey);
  expect(journey.verified, [journey.authority]);

  await journey.show(tester, isActive: false);
  expect(journey.testbed.probe.deactivations, contains(first));
  await journey.waitForRevocation(tester);
  expect(journey.revoked, [
    journey.authority,
  ], reason: 'Controller relinquish retained stale decoded HLS readiness.');

  final replacement = await _present(tester, journey);
  _expectReplacement(journey, first, replacement);
  expectNoPlaybackError(tester);
  _reportEvidence(journey, first, replacement);
}

Future<PlaybackSession> _present(
  WidgetTester tester,
  DeviceHlsReactivationJourney journey,
) async {
  final focus = await journey.show(tester, isActive: true);
  await journey.waitForFrame(tester, focus);
  await journey.testbed.waitForPlaying(tester, focus);
  return journey.testbed.probe.presentationFor(focus)!.session;
}

void _expectReplacement(
  DeviceHlsReactivationJourney journey,
  PlaybackSession first,
  PlaybackSession replacement,
) {
  expect(replacement.deliveryId, journey.authority.deliveryId);
  expect(replacement.generation, greaterThan(first.generation));
  expect(journey.verified, [journey.authority, journey.authority]);
  expect(journey.revoked, [journey.authority]);
  expect(journey.testbed.server.requestsFor('index.m3u8'), greaterThan(0));
}

void _reportEvidence(
  DeviceHlsReactivationJourney journey,
  PlaybackSession first,
  PlaybackSession replacement,
) {
  debugPrint(
    'WARP_HLS_REACTIVATE delivery=${journey.authority.deliveryId.value} '
    'representation=${journey.authority.representationId.value} '
    'revision=${journey.authority.assetRevision.value} '
    'generations=${first.generation}/${replacement.generation} '
    'decoded=${journey.verified.length} revoked=${journey.revoked.length} '
    "manifests=${journey.testbed.server.requestsFor('index.m3u8')}",
  );
}
