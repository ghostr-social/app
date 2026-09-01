part of 'warp_mixed_feed_readiness_scenario.dart';

typedef _PreparedHlsEvidence = ({
  WarpHlsLeaseEvidence lease,
  VideoPlayerController controller,
  _HlsRequestEvidence requests,
});

Future<_PreparedHlsEvidence> _waitForPreparedHls(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackDeliveryId deliveryId,
) async {
  await _waitUntil(tester, runtime, () {
    final origin = runtime.resources.origin;
    return _isPlayerReady(runtime, 1) &&
        runtime.hlsGateway.activeFor(deliveryId).length == 1 &&
        _hasPreparedHlsRequests(origin);
  });
  expect(_hasPresented(runtime, 1), isFalse, reason: _evidence(runtime));
  final origin = runtime.resources.origin;
  final requests = _hlsRequestEvidence(origin);
  _expectSelectedHlsRequests(runtime, requests);
  return (
    lease: runtime.hlsGateway.activeFor(deliveryId).single,
    controller: _hlsController(tester, runtime, deliveryId),
    requests: requests,
  );
}

void _expectPreparedHlsHandoff(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  _PreparedHlsEvidence prepared,
  WarpHlsLeaseEvidence active,
) {
  expect(identical(active, prepared.lease), isTrue, reason: _evidence(runtime));
  expect(
    identical(
      _hlsController(tester, runtime, active.deliveryId),
      prepared.controller,
    ),
    isTrue,
    reason: _evidence(runtime),
  );
  expect(
    runtime.hlsGateway.acquisitions.where(
      (item) => item.deliveryId == active.deliveryId,
    ),
    hasLength(1),
    reason: _evidence(runtime),
  );
  final origin = runtime.resources.origin;
  final requests = _hlsRequestEvidence(origin);
  _expectSelectedHlsRequests(runtime, requests);
  expect(requests, prepared.requests, reason: _evidence(runtime));
}

VideoPlayerController _hlsController(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackDeliveryId deliveryId,
) {
  final post = (runtime.graph.cubit.state as FeedLoaded).posts.singleWhere(
    (item) => item.media.playbackDeliveryId == deliveryId,
  );
  final card = find.byWidgetPredicate(
    (widget) => widget is FeedCard && widget.post.id == post.id,
    skipOffstage: false,
  );
  final player = find.descendant(
    of: card,
    matching: find.byType(VideoPlayer, skipOffstage: false),
    skipOffstage: false,
  );
  expect(player, findsOneWidget, reason: _evidence(runtime));
  return tester.widget<VideoPlayer>(player).controller;
}
