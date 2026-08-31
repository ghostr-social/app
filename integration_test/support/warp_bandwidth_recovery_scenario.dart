import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'warp_evidence_models.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_recovery_frontier.dart';

part 'warp_bandwidth_recovery_scenario_open.dart';
part 'warp_bandwidth_recovery_scenario_baseline.dart';
part 'warp_bandwidth_recovery_scenario_impair.dart';
part 'warp_bandwidth_recovery_scenario_traverse.dart';
part 'warp_bandwidth_recovery_scenario_recover.dart';
part 'warp_bandwidth_recovery_scenario_pairing.dart';
part 'warp_bandwidth_recovery_scenario_recovery_pairing.dart';
part 'warp_bandwidth_recovery_scenario_recovery_swipe.dart';
part 'warp_bandwidth_recovery_scenario_recovery_activation.dart';
part 'warp_bandwidth_recovery_scenario_recovery_evidence.dart';
part 'warp_bandwidth_recovery_scenario_recovery_response.dart';
part 'warp_bandwidth_recovery_scenario_origin_acceptance.dart';
part 'warp_bandwidth_recovery_scenario_acceptance.dart';

typedef _PacedFeed = ({
  WarpFeedPlaybackJourney journey,
  PlaybackFocus startup,
  BigInt focusGeneration,
  ProgressiveOriginBandwidthTrigger lossTrigger,
  ProgressiveOriginLinkProfile fastProfile,
  WarpDecisionRecord baselineDecision,
  int baselinePlanRevision,
});

typedef _ImpairedFeed = ({
  ProgressiveOriginLinkProfile profile,
  ProgressiveOriginLinkWindow window,
  WarpDecisionRecord decision,
  WarpReadyWindow ready,
});

typedef _RecoveryFocus = ({
  PlaybackFocus focus,
  WarpDecisionRecord decision,
  int planRevision,
  WarpReadyWindow ready,
  WarpRecoveryFrontier frontier,
  ProgressiveOriginBandwidthTrigger recoveryTrigger,
});

Future<void> runWarpBandwidthRecoveryScenario(WidgetTester tester) async {
  final opened = await _openPacedFeed(tester);
  final impaired = await _impairSharedLink(tester, opened);
  final recovery = await _traverseImpairedFeed(tester, opened, impaired);
  await _recoverSharedLink(tester, opened, impaired, recovery);
}

Future<void> runWarpBandwidthWarmReturnScenario(WidgetTester tester) async {
  final opened = await _openPacedFeed(tester);
  final impaired = await _impairSharedLink(tester, opened);
  await _traverseImpairedWarmReturn(tester, opened, impaired);
}
