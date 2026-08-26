part of 'device_playback_probe.dart';

final class PlaybackFocus {
  const PlaybackFocus(
    this.videoId,
    this.startedAt,
    this.sequence,
    this.cause,
    this.rescue,
  );

  final PlaybackVideoId videoId;
  final Duration startedAt;
  final int sequence;
  final FeedFocusCause cause;
  final FeedTransportRescue? rescue;
}

final class TimedPlaybackObservation {
  const TimedPlaybackObservation(this.elapsed, this.observation, this.sequence);

  final Duration elapsed;
  final PlaybackObservation observation;
  final int sequence;
}

final class _PlaybackSessionWindow {
  const _PlaybackSessionWindow(this.session, this.openedAt, this.closedAt);

  final PlaybackSession session;
  final int openedAt;
  final int? closedAt;

  bool contains(int sequence) {
    return sequence > openedAt && (closedAt == null || sequence < closedAt!);
  }
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
