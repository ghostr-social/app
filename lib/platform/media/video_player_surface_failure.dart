part of 'video_player_playback_port.dart';

extension _VideoPlayerSurfaceFailure on _VideoPlayerSurfaceState {
  Future<void> _handlePreparationFailure(
    VideoPlayerController controller,
    Object error,
    StackTrace stackTrace,
  ) async {
    _logInitializationFailure(error, stackTrace);
    final decoderUnsupported = _decoderUnsupported(error);
    if (decoderUnsupported || _awaitsHlsTransportRescue) {
      _failPreparation(
        decoderUnsupported
            ? PlayerPreparationFailureKind.decoderUnsupported
            : PlayerPreparationFailureKind.initialization,
      );
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
    final rejection = decoderUnsupported || _awaitsHlsTransportRescue
        ? _rejectControllerForCapability(controller)
        : _rejectController(controller);
    _lifecycle.track(rejection);
  }
}

bool _decoderUnsupported(Object error) =>
    error is PlatformException && error.code == 'VideoDecoderUnsupported';

bool _decoderUnsupportedDescription(String? description) =>
    description?.startsWith('[VideoDecoderUnsupported]') ?? false;
