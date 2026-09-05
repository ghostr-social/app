part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureDiagnostics on _WarpCachePressureDriver {
  Future<void> _reportFailure() async {
    final state = graph.cubit.state as FeedLoaded;
    final delivery = state.posts[state.activeIndex].media.playbackDeliveryId!;
    debugPrint(
      'WARP_PRESSURE_FAILURE index=${state.activeIndex} '
      'delivery=${delivery.value} capacity=${videoPlaybackCapacityOf(graph.playback)} '
      'storage=${(await _cacheCoverage()).byDelivery}',
    );
    debugPrint('WARP_PRESSURE_DELIVERY ${graph.deliveryProbe.evidence}');
    for (final stage in graph.playerStages.attemptsFor(delivery)) {
      debugPrint(
        'WARP_PRESSURE_PLAYER authority=${stage.authority} '
        'prepare=${stage.preparedAt} init=${stage.initializedAt} '
        'frame=${stage.firstFrameAt} failed=${stage.failedAt} '
        'released=${stage.releasedAt}',
      );
    }
    await _reportDecisions();
  }

  Future<void> _reportDecisions() async {
    final decisions = (await graph.evidence.decisions()).records;
    for (final decision in decisions.where((item) => item.selected != null)) {
      debugPrint(
        'WARP_PRESSURE_DECISION sequence=${decision.sequence} '
        'selected=${decision.selected?.postId} '
        'outcome=${decision.outcome.status} '
        'bytes=${decision.outcome.bytes}',
      );
    }
    for (final request in origin.requests) {
      debugPrint(
        'WARP_PRESSURE_REQUEST ${request.method}:${request.path} '
        'range=${request.range} bytes=${request.servedBytes} '
        'outcome=${request.outcome.name}',
      );
    }
  }
}
