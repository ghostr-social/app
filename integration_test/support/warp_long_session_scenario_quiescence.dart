part of 'warp_long_session_scenario.dart';

extension _WarpLongSessionQuiescence on _WarpLongSessionDriver {
  Future<int> _latestPlanRevision() async {
    final page = await graph.evidence.page();
    return page.planPage.latestRetainedRevision;
  }

  Future<void> _waitForStableQuiescence(int baseline) async {
    final timeout =
        playbackControllerTeardownTimeout + const Duration(seconds: 2);
    await _wait(_isQuiescent, timeout: timeout, awaiting: 'quiescence');
    await _waitForNativeRelease(baseline, timeout);
    await _pumpFor(const Duration(seconds: 1));
    expect(
      _isQuiescent(),
      isTrue,
      reason: _timeoutEvidence(timeout, 'quiescence'),
    );
  }

  Future<void> _waitForNativeRelease(int baseline, Duration timeout) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < timeout) {
      final page = await graph.evidence.page(afterRevision: baseline);
      final released = page.planPage.records.any(
        (plan) =>
            plan.revision > baseline && plan.playerVerifiedPostIds.isEmpty,
      );
      if (released) return;
      await _tick();
    }
    fail(
      'Native player preparations did not release after revision $baseline.',
    );
  }
}
