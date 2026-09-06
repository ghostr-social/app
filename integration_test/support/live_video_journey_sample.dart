part of 'live_video_journey.dart';

extension LiveVideoJourneySample on LiveVideoJourney {
  Future<void> sample(PlaybackFocus focus, String scenario) async {
    final probe = runtime.telemetry.probe;
    final observationStart = probe.observations.length;
    final ready = await waitUntil(() => _renderedAndMoving(focus));
    final motion = ready ? await observeMotion(focus) : null;
    final latency = probe.firstFrameLatency(focus);
    final target = scenario == 'startup'
        ? deviceStartupTarget
        : deviceFocusSwitchTarget;
    final result = <String, Object?>{
      'scenario': scenario,
      'eventId': focus.videoId.value,
      'sequence': focus.sequence,
      'deliveryId': runtime.focus.probe
          .deliveryForEvent(focus.videoId.value)
          ?.value,
      'url': runtime.focus.posts[focus.videoId.value]?.media.remoteUrl,
      'renderedAndMoving': ready,
      if (motion != null) ...motion.report(),
      'firstFrameMs': latency?.inMilliseconds,
      'targetMs': target.inMilliseconds,
      'lastPositionMs': probe.latestPositionFor(focus)?.inMilliseconds,
      'focusChanged': currentFocus?.videoId != focus.videoId,
      'unavailableVisible': find
          .text('Video unavailable')
          .evaluate()
          .isNotEmpty,
      'observations': probe.observations
          .skip(observationStart)
          .where((e) => e.observation.videoId == focus.videoId)
          .map(_observation)
          .toList(),
    };
    samples.add(result);
    log.add('sample', {...result}..remove('observations'));
    if (!ready || latency == null || latency >= target) {
      failures.add(
        '$scenario ${focus.videoId.value}: first frame=$latency, moving=$ready',
      );
    }
    if (result['unavailableVisible'] == true) {
      failures.add('$scenario ${focus.videoId.value}: Video unavailable.');
    }
    if (motion != null &&
        (motion.advances < 2 ||
            motion.longestFreeze > const Duration(seconds: 2))) {
      failures.add(
        '$scenario ${focus.videoId.value}: playback froze for ${motion.longestFreeze}.',
      );
    }
  }

  bool _renderedAndMoving(PlaybackFocus focus) {
    final probe = runtime.telemetry.probe;
    final position = probe.latestPositionFor(focus);
    return probe.firstFrameLatency(focus) != null &&
        position != null &&
        position >= const Duration(milliseconds: 500) &&
        probe.hasPhaseFor(focus, PlaybackPhase.playing);
  }

  Map<String, Object?> _observation(TimedPlaybackObservation e) => {
    'elapsedMs': e.elapsed.inMilliseconds,
    'generation': e.observation.session.generation,
    'phase': e.observation.phase.name,
    'positionMs': e.observation.position.inMilliseconds,
    'bufferAheadMs': e.observation.metrics.bufferAhead.inMilliseconds,
  };
}
