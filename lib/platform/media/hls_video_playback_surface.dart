part of 'hls_video_playback_port.dart';

final class _HlsVideoPlaybackSurface extends StatefulWidget {
  const _HlsVideoPlaybackSurface({required this.port, required this.request});

  final HlsVideoPlaybackPort port;
  final VideoPlaybackSurfaceRequest request;

  VideoMediaSource get media => request.media;
  PlaybackVideoId? get videoId => request.videoId;
  bool get isActive => request.isActive;
  VideoPlaybackMode get mode => request.mode;
  VoidCallback? get onPlaybackMediaReleased => request.onPlaybackMediaReleased;

  @override
  State<_HlsVideoPlaybackSurface> createState() =>
      _HlsVideoPlaybackSurfaceState();
}

final class _HlsVideoPlaybackSurfaceState
    extends State<_HlsVideoPlaybackSurface> {
  ProxiedHlsVideoMediaSource? _playbackMedia;
  HlsPlaybackLease? _lease;
  final Set<HlsPlaybackLease> _renderedLeases = {};
  bool _isPreparing = true;
  bool _hasPreparationError = false;
  int _requestVersion = 0;

  @override
  void initState() {
    super.initState();
    _requestGateway();
  }

  @override
  void didUpdateWidget(covariant _HlsVideoPlaybackSurface oldWidget) {
    super.didUpdateWidget(oldWidget);
    final mediaChanged =
        oldWidget.media.inventoryPlaybackIdentity !=
        widget.media.inventoryPlaybackIdentity;
    final authorityChanged =
        oldWidget.request.hlsAuthority != widget.request.hlsAuthority;
    if (mediaChanged || authorityChanged) {
      _resetMedia();
    }
  }

  @override
  void dispose() {
    _requestVersion += 1;
    _retireLease();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_hasPreparationError) return _buildError();
    final media = _playbackMedia;
    if (_isPreparing || media == null) return _buildLoading();
    return _buildPlaybackSurface(media);
  }

  Widget _buildLoading() {
    final label = widget.isActive ? 'Loading video' : 'Preparing next video';
    return VideoLoadingSurface(label: label, preview: widget.request.preview);
  }

  Widget _buildError() {
    return AsyncStatePanel(
      icon: Icons.play_disabled_outlined,
      title: 'Video unavailable',
      message: 'Ghostr could not safely prepare this stream.',
      actionLabel: 'Retry',
      onAction: _retry,
    );
  }

  void _resetMedia() {
    _requestVersion += 1;
    _retireLease();
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
      final request = HlsPlaybackRequest.fromMedia(
        widget.media,
        expectedAuthority: widget.request.hlsAuthority,
      );
      final lease = await widget.port._gateway.acquire(request);
      if (!_isCurrent(version)) {
        lease.release();
        return;
      }
      if (!_matchesRequest(lease, request)) {
        lease.release();
        throw StateError('HLS lease authority mismatch.');
      }
      _acceptMedia(lease);
    } catch (error, stackTrace) {
      _logFailure(error, stackTrace);
      if (_isCurrent(version)) _rejectMedia();
    }
  }

  bool _isCurrent(int version) => mounted && version == _requestVersion;

  void _acceptMedia(HlsPlaybackLease lease) {
    _retireLease();
    setState(() {
      _lease = lease;
      _playbackMedia = lease.media;
      _isPreparing = false;
      _hasPreparationError = false;
    });
  }

  void _rejectMedia() {
    _retireLease();
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

bool _matchesRequest(HlsPlaybackLease lease, HlsPlaybackRequest request) {
  if (lease.deliveryId != request.deliveryId) return false;
  final expected = request.expectedAuthority;
  return expected == null || lease.authority == expected;
}

void _logFailure(Object error, StackTrace stackTrace) {
  log(
    'Secure HLS gateway acquisition failed.',
    name: 'ghostr.video.hls',
    error: error,
    stackTrace: stackTrace,
  );
}
