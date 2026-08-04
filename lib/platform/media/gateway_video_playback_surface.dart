part of 'gateway_video_playback_port.dart';

final class _GatewayVideoPlaybackSurface extends StatefulWidget {
  const _GatewayVideoPlaybackSurface({
    required this.port,
    required this.media,
    required this.isActive,
    required this.onPlaybackMediaReleased,
  });

  final GatewayVideoPlaybackPort port;
  final VideoMediaSource media;
  final bool isActive;
  final void Function()? onPlaybackMediaReleased;

  @override
  State<_GatewayVideoPlaybackSurface> createState() =>
      _GatewayVideoPlaybackSurfaceState();
}

final class _GatewayVideoPlaybackSurfaceState
    extends State<_GatewayVideoPlaybackSurface> {
  ProxiedProgressiveVideoMediaSource? _playbackMedia;
  bool _isPreparing = true;
  bool _hasPreparationError = false;
  int _requestVersion = 0;

  @override
  void initState() {
    super.initState();
    _requestGateway();
  }

  @override
  void didUpdateWidget(covariant _GatewayVideoPlaybackSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.media.inventoryPlaybackIdentity !=
        widget.media.inventoryPlaybackIdentity) {
      _resetMedia();
    }
  }

  @override
  void dispose() {
    _requestVersion += 1;
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_hasPreparationError) return _buildError();
    final media = _playbackMedia;
    if (_isPreparing || media == null) return _buildLoading();
    return widget.port._delegate.buildSurface(
      media: media,
      isActive: widget.isActive,
      onPlaybackMediaReleased: widget.onPlaybackMediaReleased,
    );
  }

  Widget _buildLoading() {
    final label = widget.isActive ? 'Loading video' : 'Preparing next video';
    return ColoredBox(
      color: AppPalette.videoLoadingBackground,
      child: LoadingPanel(label: label),
    );
  }

  Widget _buildError() {
    return AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Video unavailable',
      message: 'Ghostr could not reach the local video gateway.',
      actionLabel: 'Retry',
      onAction: _retry,
    );
  }

  void _resetMedia() {
    _requestVersion += 1;
    _playbackMedia = null;
    _isPreparing = true;
    _hasPreparationError = false;
    _requestGateway();
  }

  void _requestGateway() {
    final version = ++_requestVersion;
    unawaited(_loadGatewayMedia(version));
  }

  Future<void> _loadGatewayMedia(int version) async {
    try {
      final media = await widget.port._gateway.resolve(widget.media);
      if (_isCurrent(version)) _acceptMedia(media);
    } catch (error, stackTrace) {
      _logFailure(error, stackTrace);
      if (_isCurrent(version)) _rejectMedia();
    }
  }

  bool _isCurrent(int version) => mounted && version == _requestVersion;

  void _acceptMedia(ProxiedProgressiveVideoMediaSource media) {
    setState(() {
      _playbackMedia = media;
      _isPreparing = false;
      _hasPreparationError = false;
    });
  }

  void _rejectMedia() {
    setState(() {
      _playbackMedia = null;
      _isPreparing = false;
      _hasPreparationError = true;
    });
  }

  void _retry() {
    setState(() {
      _isPreparing = true;
      _hasPreparationError = false;
    });
    _requestGateway();
  }
}

void _logFailure(Object error, StackTrace stackTrace) {
  log(
    'Progressive gateway resolution failed.',
    name: 'ghostr.video.gateway',
    error: error,
    stackTrace: stackTrace,
  );
}

final class _UnsupportedStreamPanel extends StatelessWidget {
  const _UnsupportedStreamPanel();

  @override
  Widget build(BuildContext context) {
    return const AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Streaming video unsupported',
      message: 'Secure HLS playback is not available yet.',
    );
  }
}
