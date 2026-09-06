part of 'live_video_journey.dart';

extension LiveVideoJourneyWait on LiveVideoJourney {
  Future<bool> waitUntil(
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    return condition();
  }

  Future<void> pumpFor(Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await tester.pump(const Duration(milliseconds: 50));
    }
  }

  Future<bool> swipe(LiveSwipeDirection direction) async {
    final page = find.byType(PageView).first;
    if (page.evaluate().isEmpty) return false;
    final before = currentFocus;
    await LiveVideoSwipe(tester, log).perform(direction, before);
    return waitUntil(
      () => currentFocus?.videoId != before?.videoId,
      timeout: const Duration(seconds: 5),
    );
  }

  PlaybackFocus? get currentFocus {
    final all = runtime.focus.probe.occurrences;
    return all.isEmpty ? null : all.last;
  }
}
