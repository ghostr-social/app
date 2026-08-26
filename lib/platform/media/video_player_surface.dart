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
  PlayerPreparationFeedbackPort get preparationFeedback =>
      dependencies.preparationFeedback;
  RenderedFirstFramePort get renderedFirstFrames =>
      dependencies.renderedFirstFrames;
  _VideoPlayerControllerBudget get controllerBudget =>
      dependencies.controllerBudget;
  _VideoPlayerPlaybackHandoff get handoff => dependencies.handoff;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  Completer<void>? _controllerSuperseded;
  late VideoMediaSource _playbackMedia = widget.media;
  late final _lifecycle = _VideoPlayerControllerLifecycle(
    _disposeAfterUnsafeCommands,
    widget.handoff.retire,
    widget.handoff.markTeardown,
    widget.recoveryPolicy.teardownTimeout,
  );
  late final _valueWatch = VideoPlayerValueListener(
    onValueChanged: _handleValueChange,
  );
  final _playbackObserver = VideoPlayerPlaybackObserver();
  final Completer<void> _closing = Completer<void>();
  PlaybackSession? _playbackSession;
  PlayerPreparationAttempt? _preparationAttempt;
  RenderedFirstFrameRegistration? _firstFrameRegistration;
  late PlaybackAssetAuthority? _playbackAuthority = widget.request.authority;
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
  int _presentationVersion = 0;
  bool _isRecoveryWindowOpen = false;
  bool _isObserving = false;
  bool _nativeFrameObserved = false;
  bool _controllerPresented = false;
  bool _presentationScheduled = false;
  bool _presentationReported = false;
  bool _isClosing = false;
  bool _isLoading = false;
  bool _loadRequested = false;
  Completer<void>? _pendingLoadCancellation;

  @override
  void initState() {
    super.initState();
    if (!_hasError) _startLoad();
  }

  @override
  void didUpdateWidget(covariant _VideoPlayerSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    _adoptRequestAuthority();
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
    widget.dependencies.releaseSurfaceKey(widget.request, widget.key);
    unawaited(_releaseWhenClosed(released));
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final surface = VideoPlayerSurfaceView(
      controller: _controller,
      hasError: _hasError,
      onRetry: _retry,
      preview: widget.request.preview,
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
    final controller = _controller;
    if (controller != null &&
        value.hasError &&
        _decoderUnsupportedDescription(value.errorDescription)) {
      _handleRuntimeFailure(controller, value);
      return;
    }
    final phaseChanged = _captureObservation(value);
    if (controller != null && value.hasError) {
      _handleRuntimeFailure(controller, value);
      return;
    }
    if (phaseChanged) setState(() {});
  }

  void _refresh(VoidCallback update) => setState(update);

  bool get _cannotObservePlayback => _isClosing || !mounted;

  bool get _hasError => _recoveryState == _VideoPlayerRecoveryState.exhausted;

  Future<void>? _disposeCurrentController() {
    final controller = _controller;
    if (controller != null) _relinquishController(controller);
    _playbackSession = null;
    return controller == null ? null : _disposeSafely(controller);
  }

  Future<void> _releaseWhenClosed(void Function()? released) async {
    final teardownProven = await _lifecycle.waitControllers();
    if (teardownProven) released?.call();
  }

  Future<void> _disposeSafely(VideoPlayerController controller) async {
    await _lifecycle.dispose(controller);
  }

  Future<void> _disposeAfterUnsafeCommands(
    VideoPlayerController controller,
  ) async {
    await widget.handoff.waitUnsafeCommands(controller);
    await widget.controllerDisposer(controller);
  }
}
