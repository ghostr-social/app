import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';

part 'warp_ready_burst_scenario_consume.dart';
part 'warp_ready_burst_scenario_open.dart';
part 'warp_ready_burst_scenario_retention.dart';
part 'warp_ready_burst_scenario_reverse.dart';

typedef _OpenedFeed = ({
  WarpFeedPlaybackJourney journey,
  PlaybackFocus startup,
  PlaybackFocus planning,
  ProgressiveRangedRequestPair parallel,
});

typedef _ReadyBurstResult = ({
  List<PlaybackFocus> focuses,
  PlaybackFocus finalFocus,
  WarpReadyWindow replenished,
});

Future<void> runWarpReadyBurstScenario(WidgetTester tester) async {
  final opened = await _openFeed(tester);
  final ready = await _waitForReady(tester, opened);
  final burst = await _consumeReady(tester, opened, ready);
  final next = await _consumeReplenished(tester, opened, ready, burst);
  await _consumeBackward(tester, opened.journey, next);
}
