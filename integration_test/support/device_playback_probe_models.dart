part of 'device_playback_probe.dart';

final class PlaybackFocus {
  const PlaybackFocus(this.videoId, this.startedAt);

  final PlaybackVideoId videoId;
  final Duration startedAt;
}

final class TimedPlaybackObservation {
  const TimedPlaybackObservation(this.elapsed, this.observation);

  final Duration elapsed;
  final PlaybackObservation observation;
}
