part of 'warp_mixed_feed_readiness_scenario.dart';

void _reportHlsState(
  WarpMixedFeedRuntime runtime,
  VideoDeliverySnapshot structural,
  String stage,
) {
  final state = runtime.graph.cubit.state as FeedLoaded;
  final post = state.posts.singleWhere(
    (item) => item.media.playbackDeliveryId == structural.deliveryId,
  );
  final projected = state.hlsAuthorityFor(post.media);
  final history = runtime.graph.deliveryProbe.observations
      .where((item) => item.snapshot.deliveryId == structural.deliveryId)
      .map(_hlsSnapshotEvidence)
      .join('|');
  debugPrint(
    'WARP_HLS_STATE stage=$stage active=${state.activeIndex} '
    'projected=${_hlsAuthority(projected)} history=$history',
  );
}

String _hlsSnapshotEvidence(WarpFeedDeliveryObservation observation) {
  final snapshot = observation.snapshot;
  return '${observation.sequence}:${snapshot.phase.name}:'
      '${snapshot.bytesPresent}:${_hlsAuthority(snapshot.hlsAuthority)}';
}
