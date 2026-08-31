part of 'warp_mixed_feed_readiness_scenario.dart';

Future<void> _unmountAndExpectHlsCleanup(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  WarpHlsLeaseEvidence lease,
) async {
  final authority = lease.authority;
  expect(authority, isNotNull, reason: _evidence(runtime));
  await tester.pumpWidget(const SizedBox.shrink());
  await _waitUntil(tester, runtime, () {
    final attempts = runtime.graph.playerStages.hlsAttemptsFor(authority!);
    return attempts.length == 1 &&
        attempts.single.releasedAt != null &&
        runtime.hlsGateway.activeFor(lease.deliveryId).isEmpty;
  });
  final evidence = runtime.graph.playerStages.hlsAttemptsFor(authority!).single;
  expect(evidence.authority, authority);
  expect(evidence.lifecycle, const [
    WarpFeedPlayerStage.initializing,
    WarpFeedPlayerStage.initialized,
    WarpFeedPlayerStage.firstFrameRendered,
    WarpFeedPlayerStage.released,
  ]);
  expect(evidence.initializingAt, isNotNull);
  expect(evidence.initializedAt, isNotNull);
  expect(evidence.firstFrameAt, isNotNull);
  expect(evidence.failedAt, isNull);
  expect(evidence.releasedAt, isNotNull);
  expect(runtime.hlsGateway.activeFor(lease.deliveryId), isEmpty);
  debugPrint(
    'WARP_HLS_CLEANUP delivery=${lease.deliveryId.value} '
    'authority=${_hlsAuthority(authority)} '
    'lifecycle=${evidence.lifecycle.map((stage) => stage.name).join("|")} '
    'activeLeases=0',
  );
}
