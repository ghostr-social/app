import 'warp_native_request_bounds.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'warp_evidence_models.dart';
import 'warp_feed_playback_journey.dart';

part 'warp_bandwidth_recovery_scenario_open.dart';
part 'warp_bandwidth_recovery_scenario_measure.dart';
part 'warp_bandwidth_recovery_scenario_acceptance.dart';

Future<void> runWarpBandwidthRecoveryScenario(WidgetTester tester) async {
  final journey = await _openPacedFeed(tester);
  final baseline = await _baseline(tester, journey);
  final loss = await _changeLink(tester, journey, (
    path: '/third.mp4',
    bandwidth: 700,
    baseline: baseline,
    advance: 2,
  ));
  final recovery = await _changeLink(tester, journey, (
    path: '/sixth.mp4',
    bandwidth: 2500,
    baseline: loss,
    advance: 3,
  ));
  expect(recovery.networkThroughputBps, greaterThan(loss.networkThroughputBps));
  await _expectBandwidthAcceptance(tester, journey);
}

Future<void> runWarpBandwidthWarmReturnScenario(WidgetTester tester) async {
  final journey = await _openPacedFeed(tester);
  await journey.waitForNativeStoreCoverage(tester, ['current']);
  final baseline = await _baseline(tester, journey);
  await _changeLink(tester, journey, (
    path: '/third.mp4',
    bandwidth: 700,
    baseline: baseline,
    advance: 2,
  ));
  final before = journey.resources.origin.bytesServed('current');
  final reverse = await journey.swipeBackward(
    tester,
    count: 2,
    afterSequence: journey.focusCursor,
  );
  await _expectMoving(tester, journey, reverse.focuses.last);
  await journey.waitForOriginQuiescence(tester, ['current']);
  expect(journey.resources.origin.bytesServed('current'), before);
  await _expectBandwidthAcceptance(tester, journey);
}
