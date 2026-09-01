part of 'warp_mixed_feed_readiness_scenario.dart';

Future<WarpHlsLeaseEvidence> _consumeHls(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  _PreparedHlsEvidence prepared,
) async {
  final cursor = _focusCursor(runtime);
  await _swipeUp(tester);
  final focus = await _waitForHlsFocus(tester, runtime, cursor);
  _expectActiveHls(runtime);
  await _waitForFrameOrRescue(tester, runtime, focus);
  final presentation = runtime.graph.telemetry.probe.presentationFor(focus);
  expect(presentation, isNotNull, reason: _evidence(runtime));
  final lease = _expectExactHlsLease(runtime, presentation!);
  await _pumpFor(tester, const Duration(milliseconds: 500));
  _expectStableHls(runtime, focus);
  _expectPreparedHlsHandoff(tester, runtime, prepared, lease);
  _reportHlsFrame(runtime, focus, presentation, lease);
  return lease;
}

void _expectActiveHls(WarpMixedFeedRuntime runtime) {
  final state = runtime.graph.cubit.state;
  expect(state, isA<FeedLoaded>(), reason: _evidence(runtime));
  final loaded = state as FeedLoaded;
  expect(loaded.activeIndex, 1, reason: _evidence(runtime));
  expect(loaded.posts[1].id.value, runtime.events[1].id);
}

WarpHlsLeaseEvidence _expectExactHlsLease(
  WarpMixedFeedRuntime runtime,
  TimedPlaybackOwnership presentation,
) {
  final post = (runtime.graph.cubit.state as FeedLoaded).posts[1];
  final deliveryId = post.media.playbackDeliveryId!;
  expect(presentation.session.videoId.value, runtime.events[1].id);
  expect(presentation.session.deliveryId, deliveryId);
  final active = runtime.hlsGateway.activeFor(deliveryId);
  expect(active, hasLength(1), reason: _evidence(runtime));
  final lease = active.single;
  expect(lease.representationId, VideoRepresentationId.forMedia(post.media));
  expect(
    lease.sourceUrls.map((url) => url.toString()),
    orderedEquals(post.media.remoteUrls),
  );
  expect(lease.sessionId, matches(RegExp(r'^[0-9a-f]{64}$')));
  return lease;
}

void _expectStableHls(WarpMixedFeedRuntime runtime, PlaybackFocus focus) {
  _expectActiveHls(runtime);
  expect(
    _firstRescueAfter(runtime, focus.sequence),
    isNull,
    reason: _evidence(runtime),
  );
  expect(runtime.graph.telemetry.probe.firstFrameLatency(focus), isNotNull);
  expect(find.text('Video unavailable'), findsNothing);
  _expectSelectedHlsRequests(
    runtime,
    _hlsRequestEvidence(runtime.resources.origin),
  );
  expect(runtime.resources.origin.hlsRequestsFor('index0.m4s'), greaterThan(0));
}

void _expectHlsAuthorityBridge(
  WarpMixedFeedRuntime runtime,
  VideoDeliverySnapshot structural,
  WarpHlsLeaseEvidence lease,
) {
  _reportHlsState(runtime, structural, 'afterThird');
  _reportHlsAuthority(runtime, structural, lease);
  expect(structural.deliveryId, lease.deliveryId);
  expect(
    structural.authority,
    isNull,
    reason: 'HLS must not reuse progressive PlaybackAssetAuthority.',
  );
  expect(
    structural.hlsAuthority,
    isNotNull,
    reason: 'Structurally startable HLS did not expose its typed authority.',
  );
  expect(
    lease.expectedAuthority,
    structural.hlsAuthority,
    reason: 'The exact structural HLS authority was not bound to playback.',
  );
  expect(lease.authority, structural.hlsAuthority);
}
