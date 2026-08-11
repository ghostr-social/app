part of 'video_player_playback_port.dart';

class _VideoPlayerSurface extends StatefulWidget {
  const _VideoPlayerSurface({
    super.key,
    required this.request,
    required this.controllerDisposer,
    required this.telemetry,
  });

  final VideoPlaybackSurfaceRequest request;
  final VideoPlayerControllerDisposer controllerDisposer;
  final PlaybackTelemetryPort telemetry;

  VideoMediaSource get media => request.media;
  PlaybackVideoId? get videoId => request.videoId;
  bool get isActive => request.isActive;
  VoidCallback? get onPlaybackMediaReleased => request.onPlaybackMediaReleased;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  late final _lifecycle = _VideoPlayerControllerLifecycle(
    widget.controllerDisposer,
  );
  late final _valueWatch = VideoPlayerValueListener(
    onValueChanged: _handleValueChange,
  );
  final _playbackObserver = VideoPlayerPlaybackObserver();
  final Completer<void> _closing = Completer<void>();
  PlaybackSession? _playbackSession;
  PlaybackPhase? _playbackPhase;
  Future<void> _playbackTail = Future<void>.value();
  int _playbackIntent = 0;
  late bool _hasError = !_isPlayableMedia(widget.media);
  bool _isObserving = false;
  bool _isClosing = false;

  @override
  void initState() {
    super.initState();
    if (!_hasError) _startLoad();
  }

  @override
  void didUpdateWidget(covariant _VideoPlayerSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.isActive != widget.isActive) _syncPlayback();
  }

  @override
  void dispose() {
    _isClosing = true;
    _closing.complete();
    _endObservation(_controller?.value);
    _valueWatch.detach();
    final released = widget.onPlaybackMediaReleased;
    final disposal = _disposeCurrentController();
    if (disposal != null) _lifecycle.track(disposal);
    unawaited(_releaseWhenClosed(released));
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final surface = VideoPlayerSurfaceView(
      controller: _controller,
      hasError: _hasError,
      onRetry: _retry,
    );
    if (_playbackPhase != PlaybackPhase.networkStalled) return surface;
    return Stack(
      fit: StackFit.expand,
      children: [surface, const _BufferingOverlay()],
    );
  }

  void _handleValueChange(VideoPlayerValue value) {
    if (_isClosing || !mounted) return;
    final phaseChanged = _captureObservation(value);
    final controller = _controller;
    if (controller != null && value.hasError) {
      _lifecycle.track(_rejectController(controller));
      return;
    }
    if (phaseChanged) setState(() {});
  }

  void _refresh(VoidCallback update) => setState(update);

  Future<void>? _disposeCurrentController() {
    final controller = _controller;
    _controller = null;
    return controller == null ? null : _disposeSafely(controller);
  }

  Future<void> _releaseWhenClosed(void Function()? released) async {
    await _lifecycle.close();
    released?.call();
  }

  Future<void> _disposeSafely(VideoPlayerController controller) {
    return _lifecycle.dispose(controller);
  }
}
