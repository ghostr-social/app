part of 'warp_offline_restart_fixture.dart';

extension WarpOfflineRestartFixtureAcceptance on WarpOfflineRestartFixture {
  Future<void> expectRelayUnavailable() async {
    Socket? socket;
    try {
      socket = await Socket.connect(
        manifest.relay.host,
        manifest.relay.port,
        timeout: const Duration(milliseconds: 500),
      );
    } on SocketException {
      // A refused connection is the expected offline fixture state.
    } on TimeoutException {
      // A timed-out connection is also unavailable.
    }
    final connected = socket != null;
    socket?.destroy();
    expect(connected, isFalse);
  }

  void expectNoStalePlayerReadiness() {
    expect(graph.focus.occurrences, isEmpty);
    expect(graph.telemetry.probe.activations, isEmpty);
    expect(graph.telemetry.probe.presentations, isEmpty);
    expect(graph.playerStages.progressiveAttemptCount, 0);
  }

  Future<void> expectNoOriginRequest() async {
    final page = await graph.evidence.page(afterRevision: 0, limit: 1);
    expect(resources.origin.origin.port, manifest.originPort);
    expect(resources.origin.isUnavailable, isTrue);
    expect(resources.origin.requests, isEmpty);
    expect(resources.origin.bodyRequestedIds, isEmpty);
    expect(page.evaluation.efficiency.requestCount, 0);
  }

  Future<void> waitForPlayerCleanup(
    WidgetTester tester,
    PlaybackFocus focus,
  ) async {
    final delivery = graph.focus.deliveryForEvent(manifest.eventId)!;
    await _waitOffline(tester, () {
      final attempts = graph.playerStages.attemptsFor(delivery);
      return attempts.isNotEmpty &&
          attempts.every((attempt) => attempt.releasedAt != null) &&
          graph.telemetry.probe.activations.length ==
              graph.telemetry.probe.deactivations.length;
    });
    for (final attempt in graph.playerStages.attemptsFor(delivery)) {
      expect(attempt.releasedAt, isNotNull);
      expect(attempt.failedAt, isNull);
    }
  }
}
