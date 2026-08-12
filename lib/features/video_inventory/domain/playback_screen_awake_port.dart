import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';

/// Hears playback phase transitions from live playback surfaces and decides
/// whether the device screen must stay awake.
///
/// Surfaces identify themselves by an opaque handle so several surfaces can
/// share one screen without releasing each other's demand.
abstract interface class PlaybackScreenAwakePort {
  void observePhase(Object surface, PlaybackPhase phase);

  void release(Object surface);
}

final class NoopPlaybackScreenAwakePort implements PlaybackScreenAwakePort {
  const NoopPlaybackScreenAwakePort();

  @override
  void observePhase(Object surface, PlaybackPhase phase) {}

  @override
  void release(Object surface) {}
}
