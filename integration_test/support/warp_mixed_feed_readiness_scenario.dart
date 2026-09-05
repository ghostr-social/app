import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/video_representation_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_delivery_probe.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_surface.dart';
import 'warp_evidence_models.dart';
import 'warp_hls_playback_gateway_probe.dart';
import 'warp_mixed_feed_runtime.dart';

part 'warp_mixed_feed_readiness_scenario_wait.dart';
part 'warp_mixed_feed_readiness_scenario_evidence.dart';
part 'warp_mixed_feed_readiness_scenario_state_evidence.dart';
part 'warp_mixed_feed_readiness_scenario_hls.dart';
part 'warp_mixed_feed_readiness_scenario_hls_preparation.dart';
part 'warp_mixed_feed_readiness_scenario_hls_requests.dart';
part 'warp_mixed_feed_readiness_scenario_hls_cleanup.dart';
part 'warp_mixed_feed_readiness_scenario_reserve.dart';

Future<void> runWarpMixedFeedReadinessScenario(WidgetTester tester) async {
  final runtime = await WarpMixedFeedRuntime.start();
  addTearDown(runtime.close);
  final heldThird = runtime.resources.origin.holdBeforeFirstBody({
    '/third.mp4',
  }, timeout: const Duration(minutes: 2));
  addTearDown(heldThird.release);
  await _loadMixedFeed(tester, runtime);
  final startup = await _waitForFocus(tester, runtime, 0);
  await _waitForNativeFrame(tester, runtime, startup);
  await _waitForHeldThird(tester, runtime, heldThird);
  final hlsDeliveryId = _hlsDeliveryId(runtime);
  final structural = await _waitForStructuralHls(
    tester,
    runtime,
    hlsDeliveryId,
  );
  final prepared = await _waitForPreparedHls(tester, runtime, hlsDeliveryId);
  _expectHeldThird(runtime, heldThird);
  await _waitForCanonicalHlsReserve(tester, runtime, startup, heldThird);
  final hlsLease = await _consumeHls(tester, runtime, prepared);
  heldThird.release();
  await _consumeThird(tester, runtime);
  _expectHlsAuthorityBridge(runtime, structural, hlsLease);
  await _unmountAndExpectHlsCleanup(tester, runtime, hlsLease);
}

Future<void> _loadMixedFeed(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
) async {
  await tester.pumpWidget(
    MaterialApp(home: WarpFeedSurface(graph: runtime.graph)),
  );
  unawaited(runtime.graph.cubit.load());
  await _waitUntil(tester, runtime, () {
    final state = runtime.graph.cubit.state;
    return state is FeedLoaded &&
        state.posts.length == 3 &&
        find.text('WARP signed current').evaluate().isNotEmpty;
  });
  expect(find.text('WARP signed current'), findsOneWidget);
}

Future<VideoDeliverySnapshot> _waitForStructuralHls(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackDeliveryId deliveryId,
) async {
  VideoDeliverySnapshot? structural;
  await _waitUntil(tester, runtime, () {
    structural = _structuralHls(runtime, deliveryId);
    return structural != null && _isPlayerReady(runtime, 1);
  });
  expect(_isPlayerReady(runtime, 1), isTrue);
  expect(_hasPresented(runtime, 1), isFalse);
  _reportHlsState(runtime, structural!, 'beforeSwipe');
  return structural!;
}
