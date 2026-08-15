part of 'video_player_playback_port.dart';

class _VideoPlayerSurface extends StatefulWidget {
  const _VideoPlayerSurface({
    super.key,
    required this.request,
    required this.dependencies,
  });

  final VideoPlaybackSurfaceRequest request;
  final _VideoPlayerSurfaceDependencies dependencies;

  VideoMediaSource get media => request.media;
  PlaybackVideoId? get videoId => request.videoId;
  bool get isActive => request.isActive;
  VideoPlaybackMode get mode => request.mode;
  ProgressivePlaybackRefreshPort? get progressiveRefresh =>
      request.progressiveRefresh;
  VoidCallback? get onPlaybackMediaReleased => request.onPlaybackMediaReleased;
  VideoPlayerControllerDisposer get controllerDisposer =>
      dependencies.controllerDisposer;
  PlaybackTelemetryPort get telemetry => dependencies.telemetry;
  PlaybackRecoveryPolicy get recoveryPolicy => dependencies.recoveryPolicy;
  PlaybackScreenAwakePort get screenAwake => dependencies.screenAwake;
  _VideoPlayerPlaybackHandoff get handoff => dependencies.handoff;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  late VideoMediaSource _playbackMedia = widget.media;
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
  late _VideoPlayerRecoveryState _recoveryState = _isPlayableMedia(widget.media)
      ? _VideoPlayerRecoveryState.ready
      : _VideoPlayerRecoveryState.exhausted;
  PlaybackRecoveryAttempt _recoveryAttempt = PlaybackRecoveryAttempt.first;
  PlaybackResumePoint _resumePoint = PlaybackResumePoint.start;
  PlaybackResumePoint _recoveryBaseline = PlaybackResumePoint.start;
  Timer? _recoveryTimer;
  int _recoveryVersion = 0;
  bool _isRecoveryWindowOpen = false;
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
    if (oldWidget.isActive != widget.isActive) {
      _handleActivityChange();
    } else if (oldWidget.mode != widget.mode) {
      _syncPlayback();
    }
  }

  @override
  void dispose() {
    _isClosing = true;
    _cancelRecovery();
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
    if (_cannotObservePlayback) return;
    _rememberPlaybackValue(value);
    final phaseChanged = _captureObservation(value);
    final controller = _controller;
    if (controller != null && value.hasError) {
      _lifecycle.track(_rejectController(controller));
      return;
    }
    if (phaseChanged) setState(() {});
  }

  void _refresh(VoidCallback update) => setState(update);

  bool get _cannotObservePlayback => _isClosing || !mounted;

  bool get _hasError => _recoveryState == _VideoPlayerRecoveryState.exhausted;

  Future<void>? _disposeCurrentController() {
    final controller = _controller;
    _controller = null;
    _playbackSession = null;
    return controller == null ? null : _disposeSafely(controller);
  }

  Future<void> _releaseWhenClosed(void Function()? released) async {
    await _lifecycle.close();
    released?.call();
  }

  Future<void> _disposeSafely(VideoPlayerController controller) async {
    await widget.handoff.release(controller);
    await _lifecycle.dispose(controller);
  }
}
