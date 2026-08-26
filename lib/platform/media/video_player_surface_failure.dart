part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceFailure on _VideoPlayerSurfaceState {
  Future<void> _handlePreparationFailure(
    VideoPlayerController controller,
    Object error,
    StackTrace stackTrace,
  ) async {
    _logInitializationFailure(error, stackTrace);
    if (_decoderUnsupported(error)) {
      _failPreparation(PlayerPreparationFailureKind.decoderUnsupported);
      await _rejectControllerForCapability(controller);
      return;
    }
    _failPreparation(PlayerPreparationFailureKind.initialization);
    await _rejectController(controller);
  }

  void _handleRuntimeFailure(
    VideoPlayerController controller,
    VideoPlayerValue value,
  ) {
    final decoderUnsupported = _decoderUnsupportedDescription(
      value.errorDescription,
    );
    final failure = decoderUnsupported
        ? PlayerPreparationFailureKind.decoderUnsupported
        : PlayerPreparationFailureKind.runtimePlayback;
    _failPreparation(failure);
    final rejection = decoderUnsupported
        ? _rejectControllerForCapability(controller)
        : _rejectController(controller);
    _lifecycle.track(rejection);
  }
}

bool _decoderUnsupported(Object error) =>
    error is PlatformException && error.code == 'VideoDecoderUnsupported';

bool _decoderUnsupportedDescription(String? description) =>
    description?.startsWith('[VideoDecoderUnsupported]') ?? false;
