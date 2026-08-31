part of 'warp_offline_restart_fixture.dart';

extension WarpOfflineRestartFixtureWait on WarpOfflineRestartFixture {
  Future<void> waitForCachedPost(WidgetTester tester) {
    return _waitOffline(tester, () => hasCachedSignedPost);
  }

  Future<PlaybackFocus> waitForFocus(WidgetTester tester) async {
    PlaybackFocus? focus;
    await _waitOffline(tester, () {
      focus = graph.focus.occurrenceAfter(manifest.eventId, 0);
      return focus != null;
    });
    return focus!;
  }

  Future<void> waitForFreshFrame(WidgetTester tester, PlaybackFocus focus) {
    return _waitOffline(tester, () => _hasFreshFrame(graph, focus));
  }

  Future<void> waitForCompleteCurrentCache(
    WidgetTester tester,
    PlaybackFocus focus,
  ) {
    final delivery = graph.focus.deliveryForEvent(manifest.eventId)!;
    return _waitOffline(tester, () {
      final promotion = _offlinePromotion(resources.origin, 'current');
      return _isCompletePromotion(promotion) &&
          graph.deliveryProbe.observations.any((item) {
            final snapshot = item.snapshot;
            return snapshot.deliveryId == delivery &&
                snapshot.authority != null &&
                snapshot.phase != VideoDeliveryPhase.failed &&
                snapshot.totalBytes == BigInt.from(promotion.totalBytes) &&
                snapshot.bytesPresent == snapshot.totalBytes;
          });
    });
  }

  Future<void> waitForDurableEventSnapshot(WidgetTester tester) {
    return _waitOffline(tester, () {
      return graph.rustProbe.settledWithEvent(manifest.eventId) &&
          warpOfflineSnapshotCommitted(storage.eventSnapshotFile, manifest);
    });
  }
}

bool _hasFreshFrame(WarpFeedProductionGraph graph, PlaybackFocus focus) {
  final presentation = graph.telemetry.probe.presentationFor(focus);
  final session = graph.telemetry.probe.sessionFor(focus);
  if (presentation == null || session == null) return false;
  return graph.playerStages
          .forPresentation(session.deliveryId, presentation.elapsed)
          ?.firstFrameAt !=
      null;
}

Future<void> _waitOffline(
  WidgetTester tester,
  bool Function() condition,
) async {
  final watch = Stopwatch()..start();
  while (!condition() && watch.elapsed < const Duration(seconds: 20)) {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  expect(condition(), isTrue, reason: 'WARP offline fixture timed out.');
}
