part of 'warp_mixed_feed_readiness_scenario.dart';

typedef _PreparedHlsEvidence = ({
  WarpHlsLeaseEvidence lease,
  VideoPlayerController controller,
  int manifestRequests,
  int initRequests,
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
        origin.hlsRequestsFor('index.m3u8') > 0 &&
        origin.hlsRequestsFor('init.mp4') > 0 &&
        origin.hlsRequestsFor('index0.m4s') > 0;
  });
  expect(_hasPresented(runtime, 1), isFalse, reason: _evidence(runtime));
  final origin = runtime.resources.origin;
  return (
    lease: runtime.hlsGateway.activeFor(deliveryId).single,
    controller: _hlsController(tester, runtime, deliveryId),
    manifestRequests: origin.hlsRequestsFor('index.m3u8'),
    initRequests: origin.hlsRequestsFor('init.mp4'),
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
  expect(origin.hlsRequestsFor('index.m3u8'), prepared.manifestRequests);
  expect(origin.hlsRequestsFor('init.mp4'), prepared.initRequests);
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
