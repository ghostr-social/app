part of 'warp_offline_restart_fixture.dart';

extension WarpOfflineRestartFixtureEvidence on WarpOfflineRestartFixture {
  String seedEvidence(PlaybackFocus focus) {
    final promotion = _offlinePromotion(resources.origin, 'current');
    final frame = graph.telemetry.probe.firstFrameLatency(focus);
    return 'WARP_OFFLINE_SEED event=${manifest.eventId} '
        'origin=${resources.origin.origin} frameUs=${frame?.inMicroseconds} '
        'unique=${promotion.uniqueBytes}/${promotion.totalBytes} '
        'duplicate=${promotion.duplicateBytes} ranges=${promotion.rangedResponses}';
  }

  Future<String> restoreEvidence(PlaybackFocus focus) async {
    final page = await graph.evidence.page(afterRevision: 0, limit: 1);
    final frame = graph.telemetry.probe.firstFrameLatency(focus);
    final delivery = graph.focus.deliveryForEvent(manifest.eventId)!;
    final attempts = graph.playerStages.attemptsFor(delivery);
    return 'WARP_OFFLINE_RESTORE event=${manifest.eventId} '
        'origin=${resources.origin.origin} frameUs=${frame?.inMicroseconds} '
        'originRequests=${resources.origin.requests.length} '
        'networkRequests=${page.evaluation.efficiency.requestCount} '
        'attempts=${attempts.length} released='
        '${attempts.where((attempt) => attempt.releasedAt != null).length}';
  }
}
