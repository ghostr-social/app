part of 'video_player_playback_port.dart';

class _VideoPlayerSurface extends StatefulWidget {
  const _VideoPlayerSurface({
    super.key,
    required this.media,
    required this.isActive,
    required this.onPlaybackMediaReleased,
    required this.controllerDisposer,
  });

  final VideoMediaSource media;
  final bool isActive;
  final void Function()? onPlaybackMediaReleased;
  final VideoPlayerControllerDisposer controllerDisposer;

  @override
  State<_VideoPlayerSurface> createState() => _VideoPlayerSurfaceState();
}

class _VideoPlayerSurfaceState extends State<_VideoPlayerSurface> {
  VideoPlayerController? _controller;
  late final _lifecycle =
      _VideoPlayerControllerLifecycle(widget.controllerDisposer);
  late final _valueWatch =
      VideoPlayerValueListener(onStateChanged: _handleValueChange);
  late bool _hasError = !_isPlayableMedia(widget.media);
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
    if (!_valueWatch.isStalled) return surface;
    return Stack(
      fit: StackFit.expand,
      children: [surface, const _BufferingOverlay()],
    );
  }

  Future<void> _loadController() async {
    final controller = _createController();
    _controller = controller;
    try {
      await controller.setLooping(true);
      await controller.initialize();
      await _acceptController(controller);
    } on Object catch (error, stackTrace) {
      log(
        'Video player initialization failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      await _rejectController(controller);
    }
  }

  VideoPlayerController _createController() {
    return _videoPlayerController(widget.media);
  }

  Future<void> _acceptController(VideoPlayerController controller) async {
    if (!_ownsController(controller)) {
      await _disposeSafely(controller);
      return;
    }
    await _applyPlayback(controller);
    if (!_ownsController(controller)) return;
    _valueWatch.attach(controller);
    setState(() {});
  }

  bool _ownsController(VideoPlayerController controller) {
    return !_isClosing && mounted && _controller == controller;
  }

  Future<void> _rejectController(VideoPlayerController controller) async {
    _valueWatch.detach();
    if (!_isClosing && mounted && _controller == controller) {
      setState(() {
        _controller = null;
        _hasError = true;
      });
    } else if (_controller == controller) {
      _controller = null;
    }
    await _disposeSafely(controller);
  }

  void _handleValueChange() {
    if (_isClosing || !mounted) return;
    final controller = _controller;
    if (controller != null && _valueWatch.hasError) {
      _lifecycle.track(_rejectController(controller));
      return;
    }
    setState(() {});
  }

  void _retry() {
    if (!_isPlayableMedia(widget.media)) return;
    setState(() {
      _hasError = false;
    });
    _startLoad();
  }

  Future<void>? _disposeCurrentController() {
    final controller = _controller;
    _controller = null;
    return controller == null ? null : _disposeSafely(controller);
  }

  void _syncPlayback() {
    final controller = _controller;
    if (controller == null || !controller.value.isInitialized) {
      return;
    }
    _lifecycle.track(_guardPlayback(controller));
  }

  Future<void> _guardPlayback(VideoPlayerController controller) async {
    try {
      await _applyPlayback(controller);
    } on Object catch (error, stackTrace) {
      log(
        'Video playback command failed.',
        name: 'ghostr.video.player',
        error: error,
        stackTrace: stackTrace,
      );
      await _rejectController(controller);
    }
  }

  Future<void> _applyPlayback(VideoPlayerController controller) async {
    if (widget.isActive) {
      await controller.play();
      return;
    }
    await controller.pause();
    await controller.seekTo(Duration.zero);
  }

  void _startLoad() => _lifecycle.track(_loadController());

  Future<void> _releaseWhenClosed(void Function()? released) async {
    await _lifecycle.close();
    released?.call();
  }

  Future<void> _disposeSafely(VideoPlayerController controller) {
    return _lifecycle.dispose(controller);
  }
}

final class _BufferingOverlay extends StatelessWidget {
  const _BufferingOverlay();

  @override
  Widget build(BuildContext context) {
    return const ColoredBox(
      color: AppPalette.videoLoadingBackground,
      child: LoadingPanel(label: 'Buffering video'),
    );
  }
}
