part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceControllerAcceptance on _VideoPlayerSurfaceState {
  Future<bool> _acceptController(VideoPlayerController controller) async {
    if (!_ownsController(controller)) {
      await _disposeSafely(controller);
      return false;
    }
    _valueWatch.attach(controller);
    if (!_ownsController(controller)) return false;
    await _schedulePlayback(controller);
    if (!_ownsController(controller)) return false;
    _refresh(() {});
    return true;
  }
}
