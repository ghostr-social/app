part of 'device_playback_probe.dart';

extension DevicePlaybackProbeQueries on DevicePlaybackProbe {
  Duration? playingLatency(PlaybackFocus focus) {
    final event = _firstPhaseAfter(
      PlaybackPhase.playing,
      focus.startedAt,
      videoId: focus.videoId,
    );
    return event == null ? null : event.elapsed - focus.startedAt;
  }

  Duration? firstFrameLatency(PlaybackFocus focus) {
    final presentation = _focusedPresentation(focus);
    return presentation == null ? null : presentation.elapsed - focus.startedAt;
  }

  TimedPlaybackOwnership? presentationFor(PlaybackFocus focus) {
    return _focusedPresentation(focus);
  }

  TimedPlaybackOwnership? _focusedPresentation(PlaybackFocus focus) {
    PlaybackSession? active;
    for (final event in _ownershipEvents) {
      if (event.sequence <= focus.sequence) continue;
      active = _nextActiveSession(event, active);
      if (_isFocusedPresentation(event, active, focus)) return event;
    }
    return null;
  }

  PlaybackSession? _nextActiveSession(
    TimedPlaybackOwnership event,
    PlaybackSession? active,
  ) {
    return switch (event.action) {
      PlaybackOwnershipAction.activate => event.session,
      PlaybackOwnershipAction.deactivate when active == event.session => null,
      PlaybackOwnershipAction.deactivate ||
      PlaybackOwnershipAction.presented => active,
    };
  }

  bool _isFocusedPresentation(
    TimedPlaybackOwnership event,
    PlaybackSession? active,
    PlaybackFocus focus,
  ) {
    return event.action == PlaybackOwnershipAction.presented &&
        active == event.session &&
        event.session.videoId == focus.videoId;
  }

  Duration? recoveryLatency(Duration releasedAt) {
    final event = _firstPhaseAfter(PlaybackPhase.playing, releasedAt);
    return event == null ? null : event.elapsed - releasedAt;
  }

  Duration latestPosition(PlaybackVideoId videoId) {
    var result = Duration.zero;
    for (final event in observations) {
      if (event.observation.videoId == videoId) {
        result = event.observation.position;
      }
    }
    return result;
  }

  bool hasPhaseAfter(PlaybackPhase phase, Duration after) {
    return _firstPhaseAfter(phase, after) != null;
  }

  TimedPlaybackObservation? _firstPhaseAfter(
    PlaybackPhase phase,
    Duration after, {
    PlaybackVideoId? videoId,
  }) {
    for (final event in observations) {
      if (event.elapsed < after || event.observation.phase != phase) continue;
      if (videoId == null || event.observation.videoId == videoId) return event;
    }
    return null;
  }
}
