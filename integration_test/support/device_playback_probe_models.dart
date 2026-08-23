part of 'device_playback_probe.dart';

final class PlaybackFocus {
  const PlaybackFocus(this.videoId, this.startedAt, this.sequence);

  final PlaybackVideoId videoId;
  final Duration startedAt;
  final int sequence;
}

final class TimedPlaybackObservation {
  const TimedPlaybackObservation(this.elapsed, this.observation);

  final Duration elapsed;
  final PlaybackObservation observation;
}

enum PlaybackOwnershipAction { activate, deactivate, presented }

final class TimedPlaybackOwnership {
  const TimedPlaybackOwnership(
    this.elapsed,
    this.session,
    this.action,
    this.sequence,
  );

  final Duration elapsed;
  final PlaybackSession session;
  final PlaybackOwnershipAction action;
  final int sequence;
}

bool _isPlaying(TimedPlaybackObservation event) {
  return event.observation.phase == PlaybackPhase.playing;
}
