part of 'warp_feed_playback_journey.dart';

typedef WarpProgressiveLoopEvidence = ({
  Duration beforeReset,
  Duration reset,
  Duration afterReset,
});

typedef _IndexedPlayback = ({int index, TimedPlaybackObservation event});

extension WarpFeedPlaybackJourneyLoopReopen on WarpFeedPlaybackJourney {
  Future<WarpProgressiveLoopEvidence> waitForProgressiveLoop(
    WidgetTester tester,
    PlaybackFocus focus, {
    required int afterSequence,
  }) async {
    final session = telemetry.probe.sessionFor(focus)!;
    WarpProgressiveLoopEvidence? evidence;
    await _wait(tester, () {
      final events = telemetry.probe.observations.where(
        (item) =>
            item.sequence > afterSequence &&
            item.observation.session == session,
      );
      evidence = _loopEvidence(events);
      return evidence != null;
    }, timeout: const Duration(seconds: 20));
    return evidence!;
  }
}

WarpProgressiveLoopEvidence? _loopEvidence(
  Iterable<TimedPlaybackObservation> observations,
) {
  final events = observations.toList(growable: false);
  final edge = _firstObservation(events, 0, _isLoopEdge);
  if (edge == null) return null;
  final reset = _firstObservation(events, edge.index + 1, _isLoopReset);
  if (reset == null) return null;
  final advanced = _firstObservation(
    events,
    reset.index + 1,
    (event) => _advancedFrom(event, reset.event),
  );
  if (advanced == null) return null;
  return (
    beforeReset: edge.event.observation.position,
    reset: reset.event.observation.position,
    afterReset: advanced.event.observation.position,
  );
}

_IndexedPlayback? _firstObservation(
  List<TimedPlaybackObservation> events,
  int start,
  bool Function(TimedPlaybackObservation) matches,
) {
  for (var index = start; index < events.length; index += 1) {
    if (matches(events[index])) return (index: index, event: events[index]);
  }
  return null;
}

bool _isLoopEdge(TimedPlaybackObservation event) {
  return event.observation.phase == PlaybackPhase.playing &&
      event.observation.position >= const Duration(seconds: 5);
}

bool _isLoopReset(TimedPlaybackObservation event) {
  return event.observation.phase == PlaybackPhase.playing &&
      event.observation.position <= const Duration(seconds: 1);
}

bool _advancedFrom(
  TimedPlaybackObservation event,
  TimedPlaybackObservation reset,
) {
  return event.observation.phase == PlaybackPhase.playing &&
      event.observation.position >=
          reset.observation.position + const Duration(milliseconds: 500);
}
