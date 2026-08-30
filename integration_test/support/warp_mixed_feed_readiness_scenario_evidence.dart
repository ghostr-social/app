part of 'warp_mixed_feed_readiness_scenario.dart';

VideoDeliverySnapshot? _structuralHls(
  WarpMixedFeedRuntime runtime,
  PlaybackDeliveryId deliveryId,
) {
  for (final item in runtime.graph.deliveryProbe.observations.reversed) {
    final snapshot = item.snapshot;
    if (snapshot.deliveryId == deliveryId &&
        snapshot.phase == VideoDeliveryPhase.startable) {
      return snapshot;
    }
  }
  return null;
}

bool _isPlayerReady(WarpMixedFeedRuntime runtime, int eventIndex) {
  final state = runtime.graph.cubit.state;
  if (state is! FeedLoaded) return false;
  final id = runtime.events[eventIndex].id;
  final post = state.posts.singleWhere((post) => post.id.value == id);
  return state.preparation.isPlayerVerified(post.media);
}

bool _hasPresented(WarpMixedFeedRuntime runtime, int eventIndex) {
  final id = runtime.events[eventIndex].id;
  return runtime.graph.telemetry.probe.presentations.any(
    (session) => session.videoId.value == id,
  );
}

int _focusCursor(WarpMixedFeedRuntime runtime) {
  final focuses = runtime.graph.focus.occurrences;
  return focuses.isEmpty ? 0 : focuses.last.sequence;
}

String _evidence(WarpMixedFeedRuntime runtime) {
  final focuses = runtime.graph.focus.occurrences.map(
    (item) => '${item.videoId.value}:${item.cause.name}',
  );
  return 'Mixed WARP timeout: state=${runtime.graph.cubit.state.runtimeType}, '
      'focus=${focuses.join('|')}, '
      'delivery=${runtime.graph.deliveryProbe.evidence}, '
      'hlsManifest=${runtime.resources.origin.hlsRequestsFor('index.m3u8')}, '
      'hlsSegment=${runtime.resources.origin.hlsRequestsFor('index0.m4s')}.';
}

void _reportHlsFrame(
  WarpMixedFeedRuntime runtime,
  PlaybackFocus focus,
  TimedPlaybackOwnership presentation,
  WarpHlsLeaseEvidence lease,
) {
  final frame = runtime.graph.telemetry.probe.firstFrameLatency(focus)!;
  final origin = runtime.resources.origin;
  debugPrint(
    'WARP_HLS_FRAME delivery=${lease.deliveryId.value} '
    'representation=${lease.representationId.value} '
    'session=${lease.sessionId} generation=${presentation.session.generation} '
    'frameUs=${frame.inMicroseconds} '
    'gatewayAcquisitions=${runtime.hlsGateway.acquisitions.length} '
    'activeLeases=${runtime.hlsGateway.activeFor(lease.deliveryId).length} '
    'manifestRequests=${origin.hlsRequestsFor('index.m3u8')} '
    'initRequests=${origin.hlsRequestsFor('init.mp4')} '
    'segment0Requests=${origin.hlsRequestsFor('index0.m4s')} '
    'rescued=${_firstRescueAfter(runtime, focus.sequence) != null}',
  );
}

void _reportHlsAuthority(
  WarpMixedFeedRuntime runtime,
  VideoDeliverySnapshot structural,
  WarpHlsLeaseEvidence lease,
) {
  debugPrint(
    'WARP_HLS_AUTH structural=${_hlsAuthority(structural.hlsAuthority)} '
    'request=${_hlsAuthority(lease.expectedAuthority)} '
    'lease=${_hlsAuthority(lease.authority)} '
    'gatewayAcquisitions=${runtime.hlsGateway.acquisitions.length}',
  );
}

String _hlsAuthority(HlsPlaybackAuthority? authority) {
  if (authority == null) return 'null';
  return '${authority.deliveryId.value}:${authority.representationId.value}:'
      '${authority.assetRevision.value}';
}
