part of 'live_video_journey.dart';

extension LiveVideoJourneyMotion on LiveVideoJourney {
  Future<LiveMotionWindow> observeMotion(PlaybackFocus focus) async {
    final motion = LiveMotionWindow();
    final clock = Stopwatch()..start();
    while (clock.elapsed < const Duration(seconds: 10)) {
      motion.record(
        clock.elapsed,
        runtime.telemetry.probe.latestPositionFor(focus),
      );
      await pumpFor(const Duration(milliseconds: 250));
    }
    motion.record(
      clock.elapsed,
      runtime.telemetry.probe.latestPositionFor(focus),
    );
    return motion;
  }
}
