part of 'warp_origin_timeout_fallback_scenario.dart';

extension _OriginTimeoutFallbackWait on _OriginTimeoutFallbackScenario {
  Future<_OriginTimeoutEvidence> waitForVerifiedFallback(
    WidgetTester tester,
  ) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < const Duration(seconds: 25)) {
      final evidence = _fallbackEvidence();
      if (evidence != null) return evidence;
      await journey.pumpFor(tester, const Duration(milliseconds: 70));
    }
    await journey.reportSchedulingEvidence();
    fail(_timeoutEvidence());
  }

  _OriginTimeoutEvidence? _fallbackEvidence() {
    if (!_gateIsHoldingPrimary()) return null;
    final primary = _singleGet('next');
    final fallback = _gets('next-rescue');
    if (primary == null || !_hasExactFallback(fallback)) return null;
    final ready = _readyNext();
    final attempts = journey.playerStages.attemptsFor(nextId);
    if (ready == null) return null;
    final stage = warpOriginTimeoutDecodedStage(attempts, ready.authority);
    if (stage == null) return null;
    return _OriginTimeoutEvidence(
      primary: primary,
      fallback: fallback,
      stage: stage,
    );
  }

  bool _gateIsHoldingPrimary() {
    return primaryGate.isReached &&
        !primaryGate.isReleased &&
        !primaryGate.timedOut;
  }

  ProgressiveOriginRequest? _singleGet(String id) {
    final requests = _gets(id);
    return requests.length == 1 ? requests.single : null;
  }

  bool _hasExactFallback(List<ProgressiveOriginRequest> requests) {
    return warpOriginTimeoutHasBoundedExactFallback(
      requests,
      objectLength: journey.resources.origin.objectLength,
    );
  }

  List<ProgressiveOriginRequest> _gets(String id) {
    return journey.resources.origin
        .requestsFor(id)
        .where((request) => request.method == 'GET')
        .toList();
  }

  WarpFeedCurrentPreparation? _readyNext() {
    if (journey.preparation.observations.isEmpty) return null;
    return journey.preparation.latest.upcoming
        .where((asset) => asset.authority.deliveryId == nextId)
        .where((asset) => asset.readiness == PlaybackPreparationReadiness.ready)
        .firstOrNull;
  }

  String _timeoutEvidence() {
    final attempts = journey.playerStages.attemptsFor(nextId);
    final ready = _readyNext();
    final primary = _gets('next');
    final fallback = _gets('next-rescue');
    return 'Origin timeout did not produce a player-verified fallback; '
        'gate=${primaryGate.isReached}/${primaryGate.isReleased}/'
        '${primaryGate.timedOut}, ready=${ready?.authority}, '
        'attempts=${attempts.map((item) => '${item.authority}:'
            '${item.firstFrameAt}:${item.isTerminal}').toList()}, '
        'primary=${primary.map((item) => '${item.servedBytes}:'
            '${item.outcome.name}').toList()}, '
        'fallback=${fallback.map((item) => '${item.range}:'
            '${item.servedBytes}:${item.outcome.name}').toList()}.';
  }
}
