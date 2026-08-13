part of 'video_player_playback_port.dart';

typedef _PlaybackOwnership = bool Function();

final class _VideoPlayerPlaybackHandoff {
  Future<void> _tail = Future<void>.value();
  VideoPlayerController? _audible;

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
    _audible = controller;
    if (previous != null && previous != controller) {
      await previous.setVolume(0);
    }
    await controller.setVolume(0);
    if (!ownsPlayback()) return;
    await controller.play();
    if (ownsPlayback()) await controller.setVolume(1);
  }

  Future<void> _deactivate(VideoPlayerController controller) async {
    if (_audible == controller) _audible = null;
    await controller.setVolume(0);
    await controller.pause();
  }

  Future<void> _release(VideoPlayerController controller) async {
    if (_audible != controller) return;
    _audible = null;
    await controller.setVolume(0);
  }

  Future<void> _schedule(Future<void> Function() operation) {
    final scheduled = _tail.then((_) => operation());
    _tail = scheduled.then<void>((_) {}, onError: (_, __) {});
    return scheduled;
  }
}
