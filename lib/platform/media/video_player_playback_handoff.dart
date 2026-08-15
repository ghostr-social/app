part of 'video_player_playback_port.dart';

typedef _PlaybackOwnership = bool Function();

final class _VideoPlayerPlaybackHandoff {
  Future<void> _tail = Future<void>.value();
  VideoPlayerController? _audible;
  bool _audibleIsReady = false;

  Future<void> activate(
    VideoPlayerController controller,
    _PlaybackOwnership ownsPlayback,
  ) {
    return _schedule(() => _activate(controller, ownsPlayback));
  }

  Future<void> deactivate(VideoPlayerController controller) {
    return _schedule(() => _deactivate(controller));
  }

  Future<void> release(VideoPlayerController controller) {
    return _schedule(() => _release(controller));
  }

  Future<void> _activate(
    VideoPlayerController controller,
    _PlaybackOwnership ownsPlayback,
  ) async {
    if (!ownsPlayback()) return;
    final previous = _audible;
    if (_canKeepPlaying(controller)) return;
    if (_needsVolumeRestore(controller)) {
      await _restoreVolume(controller, ownsPlayback);
      return;
    }
    _audible = controller;
    _audibleIsReady = false;
    if (previous != null && previous != controller) {
      await previous.setVolume(0);
    }
    await controller.setVolume(0);
    if (!ownsPlayback()) return;
    await controller.play();
    await _restoreVolume(controller, ownsPlayback);
  }

  Future<void> _deactivate(VideoPlayerController controller) async {
    if (_audible == controller) _clearAudible();
    await controller.setVolume(0);
    await controller.pause();
  }

  Future<void> _release(VideoPlayerController controller) async {
    if (_audible != controller) return;
    _clearAudible();
    await controller.setVolume(0);
  }

  bool _canKeepPlaying(VideoPlayerController controller) {
    return _audible == controller &&
        _audibleIsReady &&
        controller.value.isPlaying;
  }

  bool _needsVolumeRestore(VideoPlayerController controller) {
    return _audible == controller && controller.value.isPlaying;
  }

  Future<void> _restoreVolume(
    VideoPlayerController controller,
    _PlaybackOwnership ownsPlayback,
  ) async {
    if (!ownsPlayback()) return;
    await controller.setVolume(1);
    if (_audible == controller && ownsPlayback()) _audibleIsReady = true;
  }

  void _clearAudible() {
    _audible = null;
    _audibleIsReady = false;
  }

  Future<void> _schedule(Future<void> Function() operation) {
    final scheduled = _tail.then((_) => operation());
    _tail = scheduled.then<void>((_) {}, onError: (_, __) {});
    return scheduled;
  }
}
