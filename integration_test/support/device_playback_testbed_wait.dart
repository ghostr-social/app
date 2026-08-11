part of 'device_playback_testbed.dart';

extension DevicePlaybackTestbedWait on DevicePlaybackTestbed {
  Future<void> waitForPlaying(WidgetTester tester, PlaybackFocus focus) {
    return waitUntil(tester, () => probe.playingLatency(focus) != null);
  }

  Future<void> waitForPosition(
    WidgetTester tester,
    PlaybackVideoId videoId,
    Duration position,
  ) {
    return waitUntil(tester, () => probe.latestPosition(videoId) >= position);
  }

  Future<void> waitForPhase(
    WidgetTester tester,
    PlaybackPhase phase, {
    Duration after = Duration.zero,
  }) {
    return waitUntil(tester, () => probe.hasPhaseAfter(phase, after));
  }

  Future<void> waitUntil(
    WidgetTester tester,
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick(tester);
    }
    if (!condition()) {
      fail('Device playback condition timed out after $timeout.');
    }
  }

  Future<void> pumpFor(WidgetTester tester, Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tick(tester);
    }
  }
}

Future<void> _tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}

void expectNoPlaybackError() {
  expect(find.text('Video unavailable'), findsNothing);
  expect(
    find.bySemanticsLabel(RegExp('Video unavailable|Retry')),
    findsNothing,
  );
}
