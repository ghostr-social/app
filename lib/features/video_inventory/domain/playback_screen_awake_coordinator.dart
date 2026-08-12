import 'dart:async';

import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_screen_awake_port.dart';
import 'package:ghostr/features/video_inventory/domain/screen_awake_port.dart';

/// Keeps the screen awake exactly while any playback surface advances media.
///
/// The screen enables once when the first surface starts demanding wakefulness
/// and disables once when the last demanding surface stops or releases.
final class PlaybackScreenAwakeCoordinator implements PlaybackScreenAwakePort {
  PlaybackScreenAwakeCoordinator(this._screen);

  final ScreenAwakePort _screen;
  final Set<Object> _demandingSurfaces = <Object>{};

  @override
  void observePhase(Object surface, PlaybackPhase phase) {
    if (phase.keepsScreenAwake) {
      _acquire(surface);
    } else {
      release(surface);
    }
  }

  @override
  void release(Object surface) {
    if (!_demandingSurfaces.remove(surface)) return;
    if (_demandingSurfaces.isEmpty) unawaited(_screen.disable());
  }

  void _acquire(Object surface) {
    final wasIdle = _demandingSurfaces.isEmpty;
    if (_demandingSurfaces.add(surface) && wasIdle) {
      unawaited(_screen.enable());
    }
  }
}
