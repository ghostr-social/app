part of 'device_playback_probe.dart';

extension DevicePlaybackProbeQueries on DevicePlaybackProbe {
  Duration? playingLatency(PlaybackFocus focus) {
    final event = phaseFor(focus, PlaybackPhase.playing);
    return event == null ? null : event.elapsed - focus.startedAt;
  }

  Duration? firstFrameLatency(PlaybackFocus focus) {
    final presentation = _focusedPresentation(focus);
    return presentation == null ? null : presentation.elapsed - focus.startedAt;
  }

  TimedPlaybackOwnership? presentationFor(PlaybackFocus focus) {
    return _focusedPresentation(focus);
  }

  PlaybackSession? sessionFor(PlaybackFocus focus) {
    return _sessionWindow(focus)?.session;
  }

  PlaybackSession? activationFor(PlaybackFocus focus) {
    final nextFocus = _nextFocusSequence(focus);
    for (final event in _ownershipEvents) {
      if (event.sequence <= focus.sequence) continue;
      if (nextFocus != null && event.sequence >= nextFocus) return null;
      if (event.action == PlaybackOwnershipAction.activate &&
          event.session.videoId == focus.videoId) {
        return event.session;
      }
    }
    return null;
  }

  bool hasPhaseFor(PlaybackFocus focus, PlaybackPhase phase) {
    final window = _sessionWindow(focus);
    if (window == null) return false;
    return observations.any(
      (event) =>
          window.contains(event.sequence) &&
          event.observation.session == window.session &&
          event.observation.phase == phase,
    );
  }

  TimedPlaybackObservation? phaseFor(PlaybackFocus focus, PlaybackPhase phase) {
    final window = _sessionWindow(focus);
    if (window == null) return null;
    for (final event in observations) {
      if (!window.contains(event.sequence)) continue;
      if (event.observation.session != window.session) continue;
      if (event.observation.phase == phase) return event;
    }
    return null;
  }

  Duration? latestPositionFor(PlaybackFocus focus) {
    final window = _sessionWindow(focus);
    if (window == null) return null;
    TimedPlaybackObservation? latest;
    for (final event in observations) {
      if (!window.contains(event.sequence)) continue;
      if (event.observation.session == window.session) latest = event;
    }
    return latest?.observation.position;
  }

  TimedPlaybackOwnership? _focusedPresentation(PlaybackFocus focus) {
    PlaybackSession? active;
    final nextFocus = _nextFocusSequence(focus);
    for (final event in _ownershipEvents) {
      if (event.sequence <= focus.sequence) continue;
      if (nextFocus != null && event.sequence >= nextFocus) return null;
      active = _nextActiveSession(event, active);
      if (_isFocusedPresentation(event, active, focus)) return event;
    }
    return null;
  }

  _PlaybackSessionWindow? _sessionWindow(PlaybackFocus focus) {
    final presentation = _focusedPresentation(focus);
    if (presentation == null) return null;
    final openedAt = _activationSequence(focus, presentation);
    if (openedAt == null) return null;
    return _PlaybackSessionWindow(
      presentation.session,
      openedAt,
      _closedSequence(focus, presentation.session, openedAt),
    );
  }

  int? _activationSequence(
    PlaybackFocus focus,
    TimedPlaybackOwnership presentation,
  ) {
    for (final event in _ownershipEvents) {
      if (event.sequence <= focus.sequence) continue;
      if (event.sequence >= presentation.sequence) break;
      if (event.action == PlaybackOwnershipAction.activate &&
          event.session == presentation.session) {
        return event.sequence;
      }
    }
    return null;
  }

  int? _closedSequence(
    PlaybackFocus focus,
    PlaybackSession session,
    int openedAt,
  ) {
    final nextFocus = _nextFocusSequence(focus);
    for (final event in _ownershipEvents) {
      if (event.sequence <= openedAt) continue;
      if (nextFocus != null && event.sequence >= nextFocus) return nextFocus;
      if (event.action == PlaybackOwnershipAction.deactivate &&
          event.session == session) {
        return event.sequence;
      }
    }
    return nextFocus;
  }

  int? _nextFocusSequence(PlaybackFocus focus) {
    for (final candidate in _focuses) {
      if (candidate.sequence <= focus.sequence) continue;
      if (candidate.videoId != focus.videoId) return candidate.sequence;
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
